#include "jacobi_unsafe_semaphore.h"
#include <pthread.h>
#include <stdatomic.h>
#include <sched.h>
#include <string.h>

// Cache line alignment for atomic counters to avoid false sharing
typedef struct {
    _Alignas(64) atomic_size_t counter;
} AlignedAtomic;

typedef struct {
    Grid *a;
    Grid *b;
    int steps;
    int row_start;
    int row_end;
    double factor;
    AlignedAtomic *s_upper;
    AlignedAtomic *s_lower;
    int is_upper;
} ThreadArgs;

static inline void wait_for_step(AlignedAtomic *counter, size_t step) {
    const int SPIN_BEFORE_YIELD = 256;
    int spin = 0;

    while (1) {
        if (atomic_load_explicit(&counter->counter, memory_order_relaxed) >= step) {
            atomic_thread_fence(memory_order_acquire);
            break;
        }

        // Spin loop hint
        #if defined(__x86_64__) || defined(_M_X64)
            __builtin_ia32_pause();
        #elif defined(__aarch64__)
            __asm__ __volatile__("yield");
        #endif

        spin++;
        if (spin >= SPIN_BEFORE_YIELD) {
            spin = 0;
            sched_yield();
        }
    }
}

static inline void update_row(
    const double *src,
    double *dst,
    int row,
    int col_start,
    int col_end,
    double factor
) {
    for (int j = col_start; j < col_end; j++) {
        int idx = row * M + j;
        double center = src[idx];
        double laplacian = src[idx + M] + src[idx - M] +
                          src[idx + 1] + src[idx - 1] - 4.0 * center;
        dst[idx] = center + factor * laplacian;
    }
}

static inline void jacobi_band(
    const double *src,
    double *dst,
    int row_start,
    int row_end,
    double factor,
    int enforce_heat_source
) {
    int center_row = N / 2;
    int center_col = M / 2;
    int center_idx = center_row * M + center_col;

    for (int i = row_start; i < row_end; i++) {
        if (enforce_heat_source && i == center_row) {
            update_row(src, dst, i, 1, center_col, factor);
            update_row(src, dst, i, center_col + 1, M - 1, factor);
            continue;
        }
        update_row(src, dst, i, 1, M - 1, factor);
    }

    if (enforce_heat_source) {
        dst[center_idx] = 100.0;
    }
}

static void *thread_upper(void *arg) {
    ThreadArgs *args = (ThreadArgs *)arg;
    AlignedAtomic *lower_ready = args->s_lower;
    AlignedAtomic *upper_signal = args->s_upper;

    double *ptr_a = args->a->data;
    double *ptr_b = args->b->data;

    for (int step = 0; step < args->steps; step++) {
        // Wait for thread 2's previous step completion
        wait_for_step(lower_ready, step);

        const double *src = (step & 1) == 0 ? ptr_a : ptr_b;
        double *dst = (step & 1) == 0 ? ptr_b : ptr_a;

        jacobi_band(src, dst, args->row_start, args->row_end, args->factor, 0);

        atomic_store_explicit(&upper_signal->counter, step + 1, memory_order_release);
    }

    return NULL;
}

static void *thread_lower(void *arg) {
    ThreadArgs *args = (ThreadArgs *)arg;
    AlignedAtomic *upper_ready = args->s_upper;
    AlignedAtomic *lower_signal = args->s_lower;

    double *ptr_a = args->a->data;
    double *ptr_b = args->b->data;

    for (int step = 0; step < args->steps; step++) {
        // Wait for thread 1's previous step completion
        wait_for_step(upper_ready, step);

        const double *src = (step & 1) == 0 ? ptr_a : ptr_b;
        double *dst = (step & 1) == 0 ? ptr_b : ptr_a;

        jacobi_band(src, dst, args->row_start, args->row_end, args->factor, 1);

        atomic_store_explicit(&lower_signal->counter, step + 1, memory_order_release);
    }

    return NULL;
}

void jacobi_step_unsafe_semaphore(Grid *a, Grid *b, int steps) {
    int mid = N / 2;
    double factor = ALPHA * DT / (DX * DX);

    AlignedAtomic s_upper = { .counter = 0 };
    AlignedAtomic s_lower = { .counter = 0 };

    ThreadArgs args_upper = {
        .a = a,
        .b = b,
        .steps = steps,
        .row_start = 1,
        .row_end = mid,
        .factor = factor,
        .s_upper = &s_upper,
        .s_lower = &s_lower,
        .is_upper = 1
    };

    ThreadArgs args_lower = {
        .a = a,
        .b = b,
        .steps = steps,
        .row_start = mid,
        .row_end = N - 1,
        .factor = factor,
        .s_upper = &s_upper,
        .s_lower = &s_lower,
        .is_upper = 0
    };

    pthread_t t1, t2;
    pthread_create(&t1, NULL, thread_upper, &args_upper);
    pthread_create(&t2, NULL, thread_lower, &args_lower);

    pthread_join(t1, NULL);
    pthread_join(t2, NULL);
}
