#ifndef JACOBI_ATOMIC_COUNTER_H
#define JACOBI_ATOMIC_COUNTER_H

#include "jacobi_common.h"

// アトミックカウンタを用いた並列ヤコビ法
void run_atomic_counter(Grid *a, Grid *b, int steps);

#endif // JACOBI_ATOMIC_COUNTER_H