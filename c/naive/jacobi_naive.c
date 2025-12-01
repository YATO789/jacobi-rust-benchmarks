#include "jacobi_naive.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
  double *src;
  double *dst;
  int start_row;
  int end_row;
} NaiveArgs;

void *naive_worker(void *arg) {
  NaiveArgs *args = (NaiveArgs *)arg;
  double *src = args->src;
  double *dst = args->dst;
  int start_row = args->start_row;
  int end_row = args->end_row;
  double factor = ALPHA * DT / (DX * DX);

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
  return NULL;
}

void jacobi_step_naive(Grid *a, Grid *b, int steps) {
  int num_threads = 4;
  pthread_t threads[num_threads];
  NaiveArgs args[num_threads];

  double *src = a->data;
  double *dst = b->data;

  int rows_per_thread = N / num_threads;

  for (int t = 0; t < steps; t++) {
    for (int i = 0; i < num_threads; i++) {
      args[i].src = src;
      args[i].dst = dst;
      args[i].start_row = i * rows_per_thread;
      args[i].end_row = (i == num_threads - 1) ? N : (i + 1) * rows_per_thread;

      pthread_create(&threads[i], NULL, naive_worker, &args[i]);
    }

    for (int i = 0; i < num_threads; i++) {
      pthread_join(threads[i], NULL);
    }

    dst[(N / 2) * M + (M / 2)] = 100.0;

    double *temp = src;
    src = dst;
    dst = temp;
  }

  if (steps % 2 == 1) {
    memcpy(a->data, b->data, N * M * sizeof(double));
  }
}
