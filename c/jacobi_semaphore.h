#ifndef JACOBI_SEMAPHORE_H
#define JACOBI_SEMAPHORE_H

#include "jacobi_common.h"

// セマフォ（アトミック同期）を用いた並列ヤコビ法
void run_safe_semaphore_optimized(Grid *a, Grid *b, int steps);

#endif // JACOBI_SEMAPHORE_H