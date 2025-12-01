#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>

// === 定数定義 (Rust版と同一) ===
#define N 1000
#define M 1000
#define TIME_STEPS 100
#define WARMUP_STEPS 10
#define DT 0.1
#define DX 1.0
#define ALPHA 0.8

#define BENCH_ITERATIONS 15
#define BENCH_WARMUP 3

// === Grid 構造体 ===
typedef struct {
    double *data;
} Grid;

// グリッドの作成と初期化
void grid_init(Grid *g) {
    // calloc で 0.0 初期化 (Rustの vec![0.0; N*M] 相当)
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

// === ヤコビ法ステップ (シングルスレッド) ===
void jacobi_step(Grid *a, Grid *b, int steps) {
    double factor = ALPHA * DT / (DX * DX);
    double *ptr_a = a->data;
    double *ptr_b = b->data;

    for (int t = 0; t < steps; t++) {
        for (int i = 1; i < N - 1; i++) {
            for (int j = 1; j < M - 1; j++) {
                int idx = i * M + j;
                // 4近傍の平均計算など
                double laplacian = ptr_a[(i + 1) * M + j] + ptr_a[(i - 1) * M + j]
                                 + ptr_a[i * M + (j + 1)] + ptr_a[i * M + (j - 1)]
                                 - 4.0 * ptr_a[idx];
                ptr_b[idx] = ptr_a[idx] + factor * laplacian;
            }
        }

        // 熱源位置を固定 (最後に1回だけ)
        ptr_b[(N / 2) * M + (M / 2)] = 100.0;

        // ポインタのスワップ (Rustの mem::swap 相当)
        double *temp = ptr_a;
        ptr_a = ptr_b;
        ptr_b = temp;
    }
    
    // 計算に使った最終結果のポインタを構造体に戻す
    // (偶数ステップなら元通り、奇数なら入れ替わっている)
    a->data = ptr_a;
    b->data = ptr_b;
}

// === 時間計測用ユーティリティ ===
double get_time_sec() {
    struct timespec ts;
    // CLOCK_MONOTONIC は Rust の Instant::now() に相当
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec * 1e-9;
}

// 比較関数 (qsort用)
int compare_doubles(const void *a, const void *b) {
    double arg1 = *(const double *)a;
    double arg2 = *(const double *)b;
    if (arg1 < arg2) return -1;
    if (arg1 > arg2) return 1;
    return 0;
}

// === ベンチマーク実行 ===
void run_benchmark(const char *name) {
    printf("%s:\n", name);

    Grid grid_a, grid_b;
    grid_init(&grid_a);
    grid_init(&grid_b);

    // ウォームアップ
    for (int i = 0; i < BENCH_WARMUP; i++) {
        jacobi_step(&grid_a, &grid_b, WARMUP_STEPS);
    }

    double times[BENCH_ITERATIONS];
    // キャッシュクリア用ダミーデータ (最適化で消されないよう volatile 推奨だが簡易的に実装)
    volatile char dummy_cache[5 * 1024 * 1024];

    for (int i = 0; i < BENCH_ITERATIONS; i++) {
        // キャッシュクリア
        memset((void*)dummy_cache, 0, sizeof(dummy_cache));

        double start = get_time_sec();
        jacobi_step(&grid_a, &grid_b, TIME_STEPS);
        double end = get_time_sec();

        double duration = end - start;
        times[i] = duration;
        printf("  試行 %2d: %.6f s\n", i + 1, duration);
    }

    // 統計計算
    qsort(times, BENCH_ITERATIONS, sizeof(double), compare_doubles);
    
    double min = times[0];
    double max = times[BENCH_ITERATIONS - 1];
    double median = times[BENCH_ITERATIONS / 2];
    double sum = 0.0;
    for(int i=0; i<BENCH_ITERATIONS; i++) sum += times[i];
    double avg = sum / BENCH_ITERATIONS;

    printf("  ---\n");
    printf("  最小値:   %.6f s\n", min);
    printf("  中央値:   %.6f s\n", median);
    printf("  平均値:   %.6f s\n", avg);
    printf("  最大値:   %.6f s\n", max);
    printf("\n");

    grid_free(&grid_a);
    grid_free(&grid_b);
}

int main() {
    printf("=== Jacobi法 2D熱方程式ベンチマーク (C言語 Single版) ===\n");
    printf("TIME_STEPS: %d, 測定回数: %d\n\n", TIME_STEPS, BENCH_ITERATIONS);

    run_benchmark("Single Thread");

    printf("\n=== ベンチマーク完了 ===\n");
    return 0;
}