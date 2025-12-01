#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "common/jacobi_common.h"
#include "semaphore/jacobi_semaphore.h"
#include "barrier/jacobi_barrier.h"
#include "omp/jacobi_omp.h"
#include "naive/jacobi_naive.h"
#include "unsafe_semaphore/jacobi_unsafe_semaphore.h"
#include "unsafe_optimized/jacobi_unsafe_optimized.h"

// シングルスレッド版
void jacobi_step_single(Grid *a, Grid *b, int steps) {
    double factor = ALPHA * DT / (DX * DX);
    double *ptr_a = a->data;
    double *ptr_b = b->data;

    for (int t = 0; t < steps; t++) {
        for (int i = 1; i < N - 1; i++) {
            for (int j = 1; j < M - 1; j++) {
                int idx = i * M + j;
                double laplacian = ptr_a[(i + 1) * M + j] + ptr_a[(i - 1) * M + j] +
                                   ptr_a[i * M + (j + 1)] + ptr_a[i * M + (j - 1)] -
                                   4.0 * ptr_a[idx];
                ptr_b[idx] = ptr_a[idx] + factor * laplacian;
            }
        }

        ptr_b[(N / 2) * M + (M / 2)] = 100.0;

        double *temp = ptr_a;
        ptr_a = ptr_b;
        ptr_b = temp;
    }

    if (steps % 2 == 1) {
        memcpy(a->data, ptr_a, N * M * sizeof(double));
    }
}

typedef void (*TestFunc)(Grid *, Grid *, int);

void run_test(const char *name, TestFunc func, int steps) {
    Grid grid_a, grid_b;
    grid_init(&grid_a);
    grid_init(&grid_b);

    func(&grid_a, &grid_b, steps);

    char filename[256];
    snprintf(filename, sizeof(filename), "c_%s.bin", name);

    if (grid_save_to_file(&grid_a, filename) == 0) {
        printf("✓ %s -> %s\n", name, filename);

        // 中心点と周辺の値を表示（デバッグ用）
        int center_idx = (N / 2) * M + (M / 2);
        printf("  中心点 [%d][%d] = %.6f\n", N/2, M/2, grid_a.data[center_idx]);

        // 4隅の値を表示
        printf("  左上 [0][0] = %.6f\n", grid_a.data[0]);
        printf("  右上 [0][%d] = %.6f\n", M-1, grid_a.data[M-1]);
        printf("  左下 [%d][0] = %.6f\n", N-1, grid_a.data[(N-1)*M]);
        printf("  右下 [%d][%d] = %.6f\n", N-1, M-1, grid_a.data[(N-1)*M + M-1]);
        printf("\n");
    } else {
        fprintf(stderr, "✗ %s: ファイル保存失敗\n", name);
    }

    grid_free(&grid_a);
    grid_free(&grid_b);
}

int main() {
    int test_steps = 100; // テスト用のステップ数

    printf("=== C実装の結果出力テスト ===\n");
    printf("ステップ数: %d\n\n", test_steps);

    run_test("single", jacobi_step_single, test_steps);
    run_test("unsafe_semaphore", jacobi_step_unsafe_semaphore, test_steps);
    run_test("safe_semaphore", run_safe_semaphore_optimized, test_steps);
    run_test("barrier", jacobi_step_barrier, test_steps);
    run_test("openmp", jacobi_step_omp, test_steps);
    run_test("unsafe_parallel", jacobi_step_unsafe_optimized, test_steps);

    printf("全ての結果ファイルを出力しました。\n");
    return 0;
}
