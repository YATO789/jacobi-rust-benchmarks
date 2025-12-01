#ifndef JACOBI_COMMON_H
#define JACOBI_COMMON_H

#include <stdlib.h>

// 定数定義
#define N 1000
#define M 1000
#define TIME_STEPS 100
#define WARMUP_STEPS 10
#define DT 0.1
#define DX 1.0
#define ALPHA 0.8

// Grid構造体
typedef struct {
    double *data;
} Grid;

// 共通関数のプロトタイプ宣言
void grid_init(Grid *g);
void grid_free(Grid *g);

#endif // JACOBI_COMMON_H