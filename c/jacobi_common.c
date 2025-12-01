#include <stdio.h>
#include <stdlib.h>
#include "jacobi_common.h"

// グリッドの初期化
void grid_init(Grid *g) {
    g->data = (double *)calloc(N * M, sizeof(double));
    if (g->data == NULL) {
        perror("Memory allocation failed");
        exit(1);
    }
    // 格子の中心に熱源を設定
    g->data[(N / 2) * M + (M / 2)] = 100.0;
}

// グリッドの解放
void grid_free(Grid *g) {
    if (g->data) {
        free(g->data);
        g->data = NULL;
    }
}