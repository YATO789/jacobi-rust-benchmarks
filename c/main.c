#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <omp.h>

#include "common/jacobi_common.h"
#include "semaphore/jacobi_semaphore.h"
#include "barrier/jacobi_barrier.h"
#include "omp/jacobi_omp.h"
#include "naive/jacobi_naive.h"

#define BENCH_ITERATIONS 15
#define BENCH_WARMUP 3

// === シングルスレッド版の実装 (比較用) ===
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

  // データ整合性のため、奇数ステップの場合はポインタをコピーしておく（簡易対応）
  if (steps % 2 == 1) {
    memcpy(a->data, ptr_a, N * M * sizeof(double));
  }
}

// === ベンチマーク測定用 ===

// 関数ポインタ型定義
typedef void (*JacobiFunc)(Grid *, Grid *, int);

double get_time_sec() {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return ts.tv_sec + ts.tv_nsec * 1e-9;
}

int compare_doubles(const void *a, const void *b) {
  double arg1 = *(const double *)a;
  double arg2 = *(const double *)b;
  if (arg1 < arg2)
    return -1;
  if (arg1 > arg2)
    return 1;
  return 0;
}

void run_benchmark(const char *name, JacobiFunc func) {
  printf("%s:\n", name);

  Grid grid_a, grid_b;
  grid_init(&grid_a);
  grid_init(&grid_b);

  // ウォームアップ
  for (int i = 0; i < BENCH_WARMUP; i++) {
    func(&grid_a, &grid_b, WARMUP_STEPS);
    struct timespec ts = {0, 100000000L}; // 100ms wait
    nanosleep(&ts, NULL);
  }

  double times[BENCH_ITERATIONS];
  // キャッシュクリア用 (約5MB)
  volatile char dummy_cache[5 * 1024 * 1024];

  for (int i = 0; i < BENCH_ITERATIONS; i++) {
    memset((void *)dummy_cache, 0, sizeof(dummy_cache));

    // グリッドを初期状態にリセット (より正確な計測のため)
    // ※毎回callocすると遅いので、値をリセットするだけに留めるか、
    //  計算時間が支配的であればそのままでも良い。ここでは簡易的に再利用。

    double start = get_time_sec();
    func(&grid_a, &grid_b, TIME_STEPS);
    double end = get_time_sec();

    times[i] = (end - start) * 1000.0; // ミリ秒に変換
    printf("  試行 %2d: %.3f ms\n", i + 1, times[i]);

    struct timespec ts = {0, 50000000L}; // 50ms wait
    nanosleep(&ts, NULL);
  }

  qsort(times, BENCH_ITERATIONS, sizeof(double), compare_doubles);

  double min = times[0];
  double median = times[BENCH_ITERATIONS / 2];
  double max = times[BENCH_ITERATIONS - 1];
  double sum = 0;
  for (int i = 0; i < BENCH_ITERATIONS; i++)
    sum += times[i];
  double avg = sum / BENCH_ITERATIONS;

  printf("  ---\n");
  printf("  最小値:   %.3f ms\n", min);
  printf("  中央値:   %.3f ms\n", median);
  printf("  平均値:   %.3f ms\n", avg);
  printf("  最大値:   %.3f ms\n", max);
  printf("\n");

  grid_free(&grid_a);
  grid_free(&grid_b);
}

int main(int argc, char *argv[]) {
  // コマンドライン引数でスレッド数を指定可能
  int num_threads = 2; // デフォルトは2スレッド
  if (argc > 1) {
    num_threads = atoi(argv[1]);
    if (num_threads < 1) {
      fprintf(stderr, "エラー: スレッド数は1以上である必要があります\n");
      return 1;
    }
  }

  // OpenMPのスレッド数を設定
  omp_set_num_threads(num_threads);

  printf("=== Jacobi法 2D熱方程式ベンチマーク (C言語 統合版) ===\n");
  printf("TIME_STEPS: %d, 測定回数: %d, スレッド数: %d\n\n", TIME_STEPS, BENCH_ITERATIONS, num_threads);

  // 1. Single Thread 実行
  run_benchmark("Single Thread", jacobi_step_single);

  // 2. Safe Semaphore Optimized 実行
  run_benchmark("Safe Semaphore", run_safe_semaphore_optimized);

  // 3. Barrier Parallel
  run_benchmark("Barrier", jacobi_step_barrier);

  // 4. OpenMP Parallel
  run_benchmark("OpenMP", jacobi_step_omp);

  printf("\n=== ベンチマーク完了 ===\n");
  return 0;
}