#include "jacobi_barrier.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// macOS does not support pthread_barrier_t natively.
// 以下は、macOSなどの環境で pthread_barrier_t がない場合の代替実装です。
#ifdef __APPLE__
#ifndef PTHREAD_BARRIER_SERIAL_THREAD
#define PTHREAD_BARRIER_SERIAL_THREAD -1
#endif
typedef struct {
  pthread_mutex_t mutex;
  pthread_cond_t cond;
  int count;
  int crossing;
  int limit;
} pthread_barrier_t;

int pthread_barrier_init(pthread_barrier_t *barrier, const void *attr,
                         unsigned int count) {
  if (count == 0)
    return -1;
  if (pthread_mutex_init(&barrier->mutex, 0) < 0)
    return -1;
  if (pthread_cond_init(&barrier->cond, 0) < 0) {
    pthread_mutex_destroy(&barrier->mutex);
    return -1;
  }
  barrier->limit = count;
  barrier->count = 0;
  barrier->crossing = 0;
  return 0;
}

int pthread_barrier_destroy(pthread_barrier_t *barrier) {
  pthread_cond_destroy(&barrier->cond);
  pthread_mutex_destroy(&barrier->mutex);
  return 0;
}

int pthread_barrier_wait(pthread_barrier_t *barrier) {
  pthread_mutex_lock(&barrier->mutex);
  barrier->count++;
  if (barrier->count >= barrier->limit) {
    barrier->crossing++;
    barrier->count = 0;
    pthread_cond_broadcast(&barrier->cond);
    pthread_mutex_unlock(&barrier->mutex);
    return PTHREAD_BARRIER_SERIAL_THREAD;
  } else {
    int crossing = barrier->crossing;
    while (crossing == barrier->crossing) {
      pthread_cond_wait(&barrier->cond, &barrier->mutex);
    }
    pthread_mutex_unlock(&barrier->mutex);
    return 0;
  }
}
#endif
// 代替実装ここまで

typedef struct {
  Grid *a;
  Grid *b;
  int steps;
  int thread_id;
  int num_threads;
  int start_row; // [start_row, end_row) の範囲を担当
  int end_row;
  pthread_barrier_t *barrier;
} BarrierArgs;

void *barrier_worker(void *arg) {
  BarrierArgs *args = (BarrierArgs *)arg;
  int steps = args->steps;
  int start_row = args->start_row;
  int end_row = args->end_row;
  double factor = ALPHA * DT / (DX * DX);

  // 初期ポインタを引数から取得
  double *src = args->a->data;
  double *dst = args->b->data;

  // 計算する行の開始と終了（境界行を除く）
  // 1行目からN-2行目までが計算対象。担当範囲の内部でクリップする。
  int r_start = (start_row < 1) ? 1 : start_row;
  int r_end = (end_row > N - 1) ? N - 1 : end_row;

  for (int t = 0; t < steps; t++) {
    // 自身の担当領域 [r_start, r_end) のみを計算
    // 隣接セル (i-1, i+1) は前ステップの結果(src)から読み取る
    for (int i = r_start; i < r_end; i++) {
      for (int j = 1; j < M - 1; j++) {
        int idx = i * M + j;
        
        // i-1 (上), i+1 (下) のデータにアクセス。
        // i=r_startのとき、i-1は隣接スレッドの領域、i=r_end-1のとき、i+1は隣接スレッドの領域
        // 全てのデータは src (前ステップの結果) から読み込む
        double laplacian = src[(i + 1) * M + j] + src[(i - 1) * M + j] +
                           src[i * M + (j + 1)] + src[i * M + (j - 1)] -
                           4.0 * src[idx];
        dst[idx] = src[idx] + factor * laplacian;
      }
    }

    // Heat source fixed at center
    int mid_row = N / 2;
    // 熱源の行が担当範囲内 (r_start <= mid_row < r_end) なら書き込む
    if (mid_row >= r_start && mid_row < r_end) {
      dst[mid_row * M + (M / 2)] = 100.0;
    }

    // 全スレッドが計算を完了するのを待つ
    pthread_barrier_wait(args->barrier);

    // ポインタのスワップ
    double *temp = src;
    src = dst;
    dst = temp;
  }

  return NULL;
}

void jacobi_step_barrier(Grid *a, Grid *b, int steps) {
  int num_threads = 2; // 比較のため2スレッドに固定
  pthread_t threads[num_threads];
  BarrierArgs args[num_threads];
  pthread_barrier_t barrier;

  // 2スレッドでバリアを初期化
  pthread_barrier_init(&barrier, NULL, num_threads);

  int mid_row = N / num_threads;

  // スレッド0 (上半分: [0, mid_row))
  args[0].a = a;
  args[0].b = b;
  args[0].steps = steps;
  args[0].thread_id = 0;
  args[0].num_threads = num_threads;
  args[0].barrier = &barrier;
  args[0].start_row = 0;
  args[0].end_row = mid_row; 

  // スレッド1 (下半分: [mid_row, N))
  args[1].a = a;
  args[1].b = b;
  args[1].steps = steps;
  args[1].thread_id = 1;
  args[1].num_threads = num_threads;
  args[1].barrier = &barrier;
  args[1].start_row = mid_row;
  args[1].end_row = N;         

  pthread_create(&threads[0], NULL, barrier_worker, &args[0]);
  pthread_create(&threads[1], NULL, barrier_worker, &args[1]);

  for (int i = 0; i < num_threads; i++) {
    pthread_join(threads[i], NULL);
  }

  pthread_barrier_destroy(&barrier);

  // ステップ数が奇数の場合、結果は b に残っているため、a にコピーし直す
  // これにより、常に a が最新結果を持つようにする
  if (steps % 2 == 1) {
    memcpy(a->data, b->data, N * M * sizeof(double));
  }
}