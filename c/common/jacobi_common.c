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

// グリッドをバイナリファイルに保存
int grid_save_to_file(const Grid *grid, const char *path) {
    FILE *fp = fopen(path, "wb");
    if (!fp) {
        perror("Failed to open file for writing");
        return -1;
    }

    // ヘッダー: N, M (4バイトずつ、リトルエンディアン)
    unsigned int n = N;
    unsigned int m = M;
    fwrite(&n, sizeof(unsigned int), 1, fp);
    fwrite(&m, sizeof(unsigned int), 1, fp);

    // データ: f64配列
    fwrite(grid->data, sizeof(double), N * M, fp);

    fclose(fp);
    return 0;
}

// バイナリファイルからグリッドを読み込み
int grid_load_from_file(Grid *grid, const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        perror("Failed to open file for reading");
        return -1;
    }

    // ヘッダー読み込み
    unsigned int n, m;
    fread(&n, sizeof(unsigned int), 1, fp);
    fread(&m, sizeof(unsigned int), 1, fp);

    if (n != N || m != M) {
        fprintf(stderr, "Grid size mismatch: expected %dx%d, got %dx%d\n", N, M, n, m);
        fclose(fp);
        return -1;
    }

    // データ読み込み
    fread(grid->data, sizeof(double), N * M, fp);

    fclose(fp);
    return 0;
}