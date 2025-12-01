#include "jacobi_unsafe_optimized.h"
#include <pthread.h>
#include <stdatomic.h>
#include <string.h>

typedef struct {
    double *a_data;
    double *b_data;
    int steps;
    int offset;     // Offset for lower thread (0 for upper, mid*M for lower)
    int rows;       // Number of rows this thread handles
    double factor;

    // Shared boundary buffers (raw pointers, no mutex)
    double *boundary_mid_minus_1;
    double *boundary_mid;

    // Atomic synchronization flags
    atomic_size_t *upper_ready;
    atomic_size_t *lower_ready;
    atomic_size_t *upper_done;
    atomic_size_t *lower_done;

    int is_upper;
    int heat_source_local_idx;  // -1 if not in this thread's region
} UnsafeThreadArgs;

static void *unsafe_thread_func(void *arg) {
    UnsafeThreadArgs *args = (UnsafeThreadArgs *)arg;

    double *src = args->a_data + args->offset;
    double *dst = args->b_data + args->offset;
    double *my_bound = args->is_upper ? args->boundary_mid_minus_1 : args->boundary_mid;
    double *peer_bound = args->is_upper ? args->boundary_mid : args->boundary_mid_minus_1;

    atomic_size_t *my_ready = args->is_upper ? args->upper_ready : args->lower_ready;
    atomic_size_t *peer_ready = args->is_upper ? args->lower_ready : args->upper_ready;
    atomic_size_t *my_done = args->is_upper ? args->upper_done : args->lower_done;
    atomic_size_t *peer_done = args->is_upper ? args->lower_done : args->upper_done;

    for (int step = 1; step <= args->steps; step++) {
        // Copy boundary row to shared buffer using fast memcpy
        if (args->is_upper) {
            // Copy last row (rows-1)
            memcpy(my_bound, src + (args->rows - 1) * M, M * sizeof(double));
        } else {
            // Copy first row (0)
            memcpy(my_bound, src, M * sizeof(double));
        }

        atomic_store_explicit(my_ready, step, memory_order_release);
        while (atomic_load_explicit(peer_ready, memory_order_acquire) < step) {
            #if defined(__x86_64__) || defined(_M_X64)
                __builtin_ia32_pause();
            #elif defined(__aarch64__)
                __asm__ __volatile__("yield");
            #endif
        }

        // Compute internal rows (lock-free)
        for (int i = 1; i < args->rows - 1; i++) {
            const double *curr_row = src + i * M;
            const double *up_row = src + (i - 1) * M;
            const double *down_row = src + (i + 1) * M;
            double *dst_row = dst + i * M;

            for (int j = 1; j < M - 1; j++) {
                double v = curr_row[j];
                double laplacian = curr_row[j + 1] + curr_row[j - 1] +
                                  down_row[j] + up_row[j] - 4.0 * v;
                dst_row[j] = v + args->factor * laplacian;
            }
        }

        // Compute boundary row using peer's buffer
        if (args->is_upper) {
            // Last row (rows-1)
            int i = args->rows - 1;
            const double *curr_row = src + i * M;
            const double *up_row = src + (i - 1) * M;
            double *dst_row = dst + i * M;

            for (int j = 1; j < M - 1; j++) {
                double v = curr_row[j];
                double down_val = peer_bound[j];  // Read from shared buffer
                double laplacian = curr_row[j + 1] + curr_row[j - 1] +
                                  down_val + up_row[j] - 4.0 * v;
                dst_row[j] = v + args->factor * laplacian;
            }
        } else {
            // First row (0)
            const double *curr_row = src;
            const double *down_row = src + M;
            double *dst_row = dst;

            for (int j = 1; j < M - 1; j++) {
                double v = curr_row[j];
                double up_val = peer_bound[j];  // Read from shared buffer
                double laplacian = curr_row[j + 1] + curr_row[j - 1] +
                                  down_row[j] + up_val - 4.0 * v;
                dst_row[j] = v + args->factor * laplacian;
            }
        }

        // Fix heat source if in this thread's region
        if (args->heat_source_local_idx >= 0) {
            dst[args->heat_source_local_idx] = 100.0;
        }

        atomic_store_explicit(my_done, step, memory_order_release);
        while (atomic_load_explicit(peer_done, memory_order_acquire) < step) {
            #if defined(__x86_64__) || defined(_M_X64)
                __builtin_ia32_pause();
            #elif defined(__aarch64__)
                __asm__ __volatile__("yield");
            #endif
        }

        // Swap buffers
        double *temp = src;
        src = dst;
        dst = temp;
    }

    return NULL;
}

void jacobi_step_unsafe_optimized(Grid *a, Grid *b, int steps) {
    int mid = N / 2;
    double factor = ALPHA * DT / (DX * DX);

    // Allocate shared boundary buffers (no mutex protection)
    double *boundary_mid_minus_1 = malloc(M * sizeof(double));
    double *boundary_mid = malloc(M * sizeof(double));
    memset(boundary_mid_minus_1, 0, M * sizeof(double));
    memset(boundary_mid, 0, M * sizeof(double));

    // Atomic synchronization flags
    atomic_size_t upper_ready = 0;
    atomic_size_t lower_ready = 0;
    atomic_size_t upper_done = 0;
    atomic_size_t lower_done = 0;

    // Determine heat source location
    int heat_row = N / 2;
    int heat_col = M / 2;
    int upper_heat_idx = (heat_row < mid) ? (heat_row * M + heat_col) : -1;
    int lower_heat_idx = (heat_row >= mid) ? ((heat_row - mid) * M + heat_col) : -1;

    UnsafeThreadArgs args_upper = {
        .a_data = a->data,
        .b_data = b->data,
        .steps = steps,
        .offset = 0,
        .rows = mid,
        .factor = factor,
        .boundary_mid_minus_1 = boundary_mid_minus_1,
        .boundary_mid = boundary_mid,
        .upper_ready = &upper_ready,
        .lower_ready = &lower_ready,
        .upper_done = &upper_done,
        .lower_done = &lower_done,
        .is_upper = 1,
        .heat_source_local_idx = upper_heat_idx
    };

    UnsafeThreadArgs args_lower = {
        .a_data = a->data,
        .b_data = b->data,
        .steps = steps,
        .offset = mid * M,
        .rows = N - mid,
        .factor = factor,
        .boundary_mid_minus_1 = boundary_mid_minus_1,
        .boundary_mid = boundary_mid,
        .upper_ready = &upper_ready,
        .lower_ready = &lower_ready,
        .upper_done = &upper_done,
        .lower_done = &lower_done,
        .is_upper = 0,
        .heat_source_local_idx = lower_heat_idx
    };

    pthread_t t1, t2;
    pthread_create(&t1, NULL, unsafe_thread_func, &args_upper);
    pthread_create(&t2, NULL, unsafe_thread_func, &args_lower);

    pthread_join(t1, NULL);
    pthread_join(t2, NULL);

    // Copy result back if odd number of steps
    if (steps % 2 == 1) {
        memcpy(a->data, b->data, N * M * sizeof(double));
    }

    free(boundary_mid_minus_1);
    free(boundary_mid);
}
