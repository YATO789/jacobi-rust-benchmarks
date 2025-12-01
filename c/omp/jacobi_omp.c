#include "jacobi_omp.h"
#include <omp.h>
#include <stdio.h>
#include <string.h>

void jacobi_step_omp(Grid *a, Grid *b, int steps) {
  double factor = ALPHA * DT / (DX * DX);
  double *src = a->data;
  double *dst = b->data;

  for (int t = 0; t < steps; t++) {
#pragma omp parallel for
    for (int i = 1; i < N - 1; i++) {
      for (int j = 1; j < M - 1; j++) {
        int idx = i * M + j;
        double laplacian = src[(i + 1) * M + j] + src[(i - 1) * M + j] +
                           src[i * M + (j + 1)] + src[i * M + (j - 1)] -
                           4.0 * src[idx];
        dst[idx] = src[idx] + factor * laplacian;
      }
    }

    // Heat source
    dst[(N / 2) * M + (M / 2)] = 100.0;

    // Swap pointers
    double *temp = src;
    src = dst;
    dst = temp;
  }

  if (steps % 2 == 1) {
    memcpy(a->data, b->data, N * M * sizeof(double));
  }
}
