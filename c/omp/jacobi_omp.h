#ifndef JACOBI_OMP_H
#define JACOBI_OMP_H

#include "jacobi_common.h"

void jacobi_step_omp(Grid *a, Grid *b, int steps);

#endif // JACOBI_OMP_H
