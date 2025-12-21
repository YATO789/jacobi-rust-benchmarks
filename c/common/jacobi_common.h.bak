#ifndef JACOBI_COMMON_H
#define JACOBI_COMMON_H

#include <stdlib.h>

// 定数定義
#define N 256
#define M 256
#define TIME_STEPS 1000
#define WARMUP_STEPS 10
#define DT 0.1
#define DX 1.0
#define ALPHA 0.8

// キャッシュラインアラインメント（64バイト）
#define CACHE_LINE_SIZE 64

// Grid構造体
typedef struct {
    double *data;
} Grid;

// 共通関数のプロトタイプ宣言
void grid_init(Grid *g);
void grid_free(Grid *g);
int grid_save_to_file(const Grid *grid, const char *path);
int grid_load_from_file(Grid *grid, const char *path);

#endif // JACOBI_COMMON_H