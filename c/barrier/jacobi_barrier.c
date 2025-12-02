#include "jacobi_barrier.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// macOS does not support pthread_barrier_t natively.
// We need to implement a simple barrier if it's not available.
#ifdef __APPLE__
#ifndef PTHREAD_BARRIER_SERIAL_THREAD
#define PTHREAD_BARRIER_SERIAL_THREAD -1
#endif
typedef struct {
  pthread_mutex_t mutex;
  pthread_cond_t cond;
  int count;
  int crossing;
  int limit;
} pthread_barrier_t;

int pthread_barrier_init(pthread_barrier_t *barrier, const void *attr,
                         unsigned int count) {
  if (count == 0)
    return -1;
  if (pthread_mutex_init(&barrier->mutex, 0) < 0)
    return -1;
  if (pthread_cond_init(&barrier->cond, 0) < 0) {
    pthread_mutex_destroy(&barrier->mutex);
    return -1;
  }
  barrier->limit = count;
  barrier->count = 0;
  barrier->crossing = 0;
  return 0;
}

int pthread_barrier_destroy(pthread_barrier_t *barrier) {
  pthread_cond_destroy(&barrier->cond);
  pthread_mutex_destroy(&barrier->mutex);
  return 0;
}

int pthread_barrier_wait(pthread_barrier_t *barrier) {
  pthread_mutex_lock(&barrier->mutex);
  barrier->count++;
  if (barrier->count >= barrier->limit) {
    barrier->crossing++;
    barrier->count = 0;
    pthread_cond_broadcast(&barrier->cond);
    pthread_mutex_unlock(&barrier->mutex);
    return PTHREAD_BARRIER_SERIAL_THREAD;
  } else {
    int crossing = barrier->crossing;
    while (crossing == barrier->crossing) {
      pthread_cond_wait(&barrier->cond, &barrier->mutex);
    }
    pthread_mutex_unlock(&barrier->mutex);
    return 0;
  }
}
#endif

typedef struct {
  Grid *a;
  Grid *b;
  int steps;
  int thread_id;
  int num_threads;
  int start_row;
  int end_row;
  pthread_barrier_t *barrier;
} BarrierArgs;

void *barrier_worker(void *arg) {
  BarrierArgs *args = (BarrierArgs *)arg;
  Grid *a = args->a;
  Grid *b = args->b;
  int steps = args->steps;
  int start_row = args->start_row;
  int end_row = args->end_row;
  double factor = ALPHA * DT / (DX * DX);

  double *src = a->data;
  double *dst = b->data;

  for (int t = 0; t < steps; t++) {
    // Compute inner points for this thread's chunk
    // We need to be careful about boundaries.
    // The loop should go from max(1, start_row) to min(N-1, end_row)

    int r_start = (start_row < 1) ? 1 : start_row;
    int r_end = (end_row > N - 1) ? N - 1 : end_row;

    for (int i = r_start; i < r_end; i++) {
      for (int j = 1; j < M - 1; j++) {
        int idx = i * M + j;
        double laplacian = src[(i + 1) * M + j] + src[(i - 1) * M + j] +
                           src[i * M + (j + 1)] + src[i * M + (j - 1)] -
                           4.0 * src[idx];
        dst[idx] = src[idx] + factor * laplacian;
      }
    }

    // Heat source fixed at center
    // Only the thread covering the center needs to set this
    int mid_row = N / 2;
    if (mid_row >= start_row && mid_row < end_row) {
      dst[mid_row * M + (M / 2)] = 100.0;
    }

    // Wait for all threads to finish computation
    pthread_barrier_wait(args->barrier);

    // Swap pointers locally? No, we need to swap globally or just swap what we
    // read/write. In this implementation, we swap src/dst pointers at the end
    // of loop. But since src/dst are local variables, we just swap them.
    // Everyone swaps in sync because of the barrier.
    double *temp = src;
    src = dst;
    dst = temp;
  }

  return NULL;
}

void jacobi_step_barrier(Grid *a, Grid *b, int steps) {
  int num_threads = 2; // Fixed to 2 threads for fair comparison
  pthread_t threads[num_threads];
  BarrierArgs args[num_threads];
  pthread_barrier_t barrier;

  pthread_barrier_init(&barrier, NULL, num_threads);

  int rows_per_thread = N / num_threads;

  // We need to ensure we are working on the correct data.
  // The worker assumes 'src' is 'a->data' initially.

  for (int i = 0; i < num_threads; i++) {
    args[i].a = a;
    args[i].b = b;
    args[i].steps = steps;
    args[i].thread_id = i;
    args[i].num_threads = num_threads;
    args[i].barrier = &barrier;

    args[i].start_row = i * rows_per_thread;
    args[i].end_row = (i == num_threads - 1) ? N : (i + 1) * rows_per_thread;

    pthread_create(&threads[i], NULL, barrier_worker, &args[i]);
  }

  for (int i = 0; i < num_threads; i++) {
    pthread_join(threads[i], NULL);
  }

  pthread_barrier_destroy(&barrier);

  // If odd steps, the valid data is in 'b', but we want it in 'a' usually?
  // The Rust implementation might swap.
  // In the worker, they swapped src/dst.
  // If steps is odd:
  // t=0: read a, write b. swap. src=b, dst=a.
  // ...
  // t=steps-1 (even): read a, write b. swap. src=b, dst=a.
  // Wait, let's trace:
  // t=0: src=a, dst=b. Write to b. Swap -> src=b, dst=a.
  // t=1: src=b, dst=a. Write to a. Swap -> src=a, dst=b.
  // ...
  // If steps is odd (e.g. 1):
  // t=0: write to b. src=b.
  // So valid data is in b.
  // We need to copy b to a if we want a to hold the result, or just leave it.
  // The single threaded version copies if steps % 2 == 1.
  if (steps % 2 == 1) {
    memcpy(a->data, b->data, N * M * sizeof(double));
  }
}
