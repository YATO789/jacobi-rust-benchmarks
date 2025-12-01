#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <stdatomic.h>

// x86系CPUでのスピンループ最適化 (PAUSE命令)
#if defined(__x86_64__) || defined(_M_X64)
#include <immintrin.h>
#define CPU_RELAX() _mm_pause()
#else
#define CPU_RELAX() ((void)0)
#endif

#include "jacobi_semaphore.h"

// スレッド引数構造体
typedef struct {
    // データポインタ
    double *src_start;
    double *dst_start;
    
    // 担当範囲
    int rows;      // 担当する行数
    int is_upper;  // 上半分か下半分か
    
    // 共有境界バッファ (Mutexの代わり)
    double *my_boundary_writer;    // 自分が書き込むバッファ
    double *other_boundary_reader; // 相手が書き込んだバッファ (読み取り用)

    // 同期用アトミックカウンターへのポインタ
    atomic_size_t *my_ready;
    atomic_size_t *other_ready;
    atomic_size_t *my_done;
    atomic_size_t *other_done;

    int steps;
} ThreadArgs;

// スレッドワーカー関数
void *worker_thread(void *arg) {
    ThreadArgs *args = (ThreadArgs *)arg;
    double *src = args->src_start;
    double *dst = args->dst_start;
    int rows = args->rows;
    double factor = ALPHA * DT / (DX * DX);

    for (int step = 1; step <= args->steps; step++) {
        // --- 1. 境界データの書き込みフェーズ ---
        
        // 境界行を共有バッファにコピー
        // Upperスレッドなら最下行、Lowerスレッドなら最上行をコピー
        if (args->is_upper) {
            // Upper: 末尾の行 (rows - 1) を書き込む
            memcpy(args->my_boundary_writer, &src[(rows - 1) * M], M * sizeof(double));
        } else {
            // Lower: 先頭の行 (0) を書き込む
            memcpy(args->my_boundary_writer, &src[0], M * sizeof(double));
        }

        // 通知: 「データ準備よし (Ready)」
        atomic_store_explicit(args->my_ready, step, memory_order_release);

        // 待機: 相手のデータ準備ができるまでスピン待機
        while (atomic_load_explicit(args->other_ready, memory_order_acquire) < step) {
            CPU_RELAX();
        }

        // --- 2. 計算フェーズ ---

        // 内部領域の計算 (1行目から rows-2行目まで)
        for (int i = 1; i < rows - 1; i++) {
            for (int j = 1; j < M - 1; j++) {
                int idx = i * M + j;
                double laplacian = src[idx - M] + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                dst[idx] = src[idx] + factor * laplacian;
            }
        }

        // 境界領域の計算 (相手のバッファを読む)
        if (args->is_upper) {
            // Upper: 最下行 (rows-1) の計算。下側の値は shared buffer から読む
            int i = rows - 1;
            for (int j = 1; j < M - 1; j++) {
                int idx = i * M + j;
                double down_val = args->other_boundary_reader[j]; // 共有バッファから読み取り
                double laplacian = src[idx - M] + down_val + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                dst[idx] = src[idx] + factor * laplacian;
            }
        } else {
            // Lower: 最上行 (0) の計算。上側の値は shared buffer から読む
            int i = 0;
            for (int j = 1; j < M - 1; j++) {
                int idx = i * M + j;
                double up_val = args->other_boundary_reader[j]; // 共有バッファから読み取り
                double laplacian = up_val + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                dst[idx] = src[idx] + factor * laplacian;
            }
        }

        // 熱源の固定 (Rustコードのロジックを再現)
        // mid = N/2。Upperは 0..mid, Lowerは mid..N
        // 熱源は (N/2, M/2) なので、Lowerの 0行目にある
        if (!args->is_upper) {
             // Lowerスレッド内でのローカル座標
            int local_row = (N / 2) - (N / 2); // 0
            dst[local_row * M + (M / 2)] = 100.0;
        }

        // 通知: 「計算完了 (Done)」
        atomic_store_explicit(args->my_done, step, memory_order_release);

        // 待機: 相手も計算を終えるまで待つ (バッファの上書き防止)
        while (atomic_load_explicit(args->other_done, memory_order_acquire) < step) {
            CPU_RELAX();
        }

        // ポインタのスワップ
        double *temp = src;
        src = dst;
        dst = temp;
    }

    // 最終的なポインタの状態を返すために引数構造体を更新する場合はここで行うが、
    // Rust版と同様、親側で一括管理するか、偶数回ステップ前提であれば無視可能。
    // ここでは簡略化のため何もしない。
    return NULL;
}

void run_safe_semaphore_optimized(Grid *a, Grid *b, int steps) {
    int mid = N / 2;

    // アトミックカウンターの確保と初期化
    atomic_size_t upper_ready = ATOMIC_VAR_INIT(0);
    atomic_size_t lower_ready = ATOMIC_VAR_INIT(0);
    atomic_size_t upper_done = ATOMIC_VAR_INIT(0);
    atomic_size_t lower_done = ATOMIC_VAR_INIT(0);

    // 境界共有バッファの確保 (Rustの boundary_mid_minus_1, boundary_mid に相当)
    double *boundary_upper = (double *)calloc(M, sizeof(double)); // Upperが書き込む
    double *boundary_lower = (double *)calloc(M, sizeof(double)); // Lowerが書き込む

    pthread_t thread_u, thread_l;
    ThreadArgs args_u, args_l;

    // --- Upper Thread 設定 ---
    args_u.src_start = a->data;           // 先頭から
    args_u.dst_start = b->data;
    args_u.rows = mid;
    args_u.is_upper = 1;
    args_u.my_boundary_writer = boundary_upper;
    args_u.other_boundary_reader = boundary_lower;
    args_u.my_ready = &upper_ready;
    args_u.other_ready = &lower_ready;
    args_u.my_done = &upper_done;
    args_u.other_done = &lower_done;
    args_u.steps = steps;

    // --- Lower Thread 設定 ---
    args_u.src_start = a->data + (mid * M); // mid行目から
    args_u.dst_start = b->data + (mid * M);
    args_l.src_start = a->data + (mid * M);
    args_l.dst_start = b->data + (mid * M);
    args_l.rows = N - mid;
    args_l.is_upper = 0;
    args_l.my_boundary_writer = boundary_lower;
    args_l.other_boundary_reader = boundary_upper;
    args_l.my_ready = &lower_ready;
    args_l.other_ready = &upper_ready;
    args_l.my_done = &lower_done;
    args_l.other_done = &upper_done;
    args_l.steps = steps;

    // スレッド作成
    pthread_create(&thread_u, NULL, worker_thread, &args_u);
    pthread_create(&thread_l, NULL, worker_thread, &args_l);

    // 終了待機
    pthread_join(thread_u, NULL);
    pthread_join(thread_l, NULL);

    // 後始末
    free(boundary_upper);
    free(boundary_lower);

    // 奇数ステップの場合、Grid a に最新データが入っているのでコピーが必要
    if (steps % 2 == 1) {
        memcpy(a->data, b->data, N * M * sizeof(double));
    }
}