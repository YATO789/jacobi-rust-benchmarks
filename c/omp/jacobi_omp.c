#include "jacobi_omp.h" // N, M, Grid, 定数などが定義されているヘッダ
#include <omp.h>
#include <stdio.h>
#include <string.h>

void jacobi_step_omp(Grid *a, Grid *b, int steps) {
  double factor = ALPHA * DT / (DX * DX);
  double *src = a->data;
  double *dst = b->data;

  for (int t = 0; t < steps; t++) {
    // === 修正点 1: 境界行のコピー (0行目とN-1行目) ===
    // Rust版と同様に、計算対象外の境界行をsrcからdstにコピーする
    memcpy(dst, src, M * sizeof(double));           // 最上行 (0行目)
    memcpy(dst + (N - 1) * M, src + (N - 1) * M, M * sizeof(double)); // 最下行 (N-1行目)

    // OpenMPによる並列計算
#pragma omp parallel for
    for (int i = 1; i < N - 1; i++) {
        // === 修正点 2: 境界列のコピー (0列目とM-1列目) ===
        // 各行の両端 (0列目とM-1列目) は計算しないため、コピーが必要
        dst[i * M] = src[i * M];           // 左端 (0列目)
        dst[i * M + M - 1] = src[i * M + M - 1]; // 右端 (M-1列目)

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

  // 奇数ステップの場合、結果を a に戻す
  if (steps % 2 == 1) {
    memcpy(a->data, src, N * M * sizeof(double)); // srcが最終的な有効データ
  }
}