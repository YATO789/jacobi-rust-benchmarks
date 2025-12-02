#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// x86系CPUでのスピンループ最適化
#if defined(__x86_64__) || defined(_M_X64)
#include <immintrin.h>
#define CPU_RELAX() _mm_pause()
#else
#define CPU_RELAX() ((void)0)
#endif

#include "jacobi_semaphore.h" // Grid定義などを想定

typedef struct {
    // 全体のベースポインタ (ここからオフセット計算する)
    double *grid_a_base;
    double *grid_b_base;

    // 自分の担当範囲 (行番号)
    int row_start;
    int row_end;

    // 同期用カウンター
    atomic_size_t *my_counter;
    atomic_size_t *other_counter;

    int steps;
    int is_lower_thread; // 熱源管理用
} ThreadArgs;

void *worker_fast(void *arg) {
    ThreadArgs *args = (ThreadArgs *)arg;
    int steps = args->steps;
    double factor = ALPHA * DT / (DX * DX);
    
    // ローカルポインタとして保持
    double *src = args->grid_a_base;
    double *dst = args->grid_b_base;

    // 担当範囲
    int r_start = args->row_start;
    int r_end = args->row_end;

    // 固定熱源の位置 (N/2, M/2)
    int heat_source_idx = (N / 2) * M + (M / 2);
    // 自分が熱源を担当するか？
    int has_heat_source = (heat_source_idx >= r_start * M && heat_source_idx < r_end * M);

    for (int step = 1; step <= steps; step++) {
        // --- 計算フェーズ (同期なし) ---
        // src(読み取り専用) から dst(書き込み専用) へ計算
        // 境界バッファへのコピーは不要。直接 src の隣接行を読む。

        for (int i = r_start; i < r_end; i++) {
            // ポインタ演算の最適化 (行の先頭ポインタ)
            double *src_curr = src + i * M;
            double *src_up   = src + (i - 1) * M;
            double *src_down = src + (i + 1) * M;
            double *dst_curr = dst + i * M;

            for (int j = 1; j < M - 1; j++) {
                double v = src_curr[j];
                double laplacian = src_curr[j + 1] 
                                 + src_curr[j - 1] 
                                 + src_down[j]   // 上下も直接読む(競合しない)
                                 + src_up[j]
                                 - 4.0 * v;
                dst_curr[j] = v + factor * laplacian;
            }
        }

        // 固定熱源
        if (has_heat_source) {
            dst[heat_source_idx] = 100.0;
        }

        // --- 同期フェーズ (1回のみ) ---
        // 「書き込み完了」を通知
        atomic_store_explicit(args->my_counter, step, memory_order_release);

        // 相手も書き込み終わるまで待つ (Spin wait)
        while (atomic_load_explicit(args->other_counter, memory_order_acquire) < (size_t)step) {
            CPU_RELAX();
        }

        // --- ポインタスワップ ---
        // 次のステップに向けて src と dst を入れ替え
        double *temp = src;
        src = dst;
        dst = temp;
    }

    return NULL;
}

void run_safe_semaphore_optimized(Grid *a, Grid *b, int steps) {
    int mid = N / 2;

    atomic_size_t count_u = 0;
    atomic_size_t count_l = 0;

    pthread_t thread_u, thread_l;
    ThreadArgs args_u, args_l;

    // 上半分スレッド (Row 1 ~ mid)
    args_u.grid_a_base = a->data;
    args_u.grid_b_base = b->data;
    args_u.row_start = 1;
    args_u.row_end = mid;
    args_u.my_counter = &count_u;
    args_u.other_counter = &count_l;
    args_u.steps = steps;
    args_u.is_lower_thread = 0;

    // 下半分スレッド (Row mid ~ N-1)
    args_l.grid_a_base = a->data;
    args_l.grid_b_base = b->data;
    args_l.row_start = mid;
    args_l.row_end = N - 1;
    args_l.my_counter = &count_l;
    args_l.other_counter = &count_u;
    args_l.steps = steps;
    args_l.is_lower_thread = 1;

    pthread_create(&thread_u, NULL, worker_fast, &args_u);
    pthread_create(&thread_l, NULL, worker_fast, &args_l);

    pthread_join(thread_u, NULL);
    pthread_join(thread_l, NULL);

    // 奇数ステップなら結果を書き戻す
    if (steps % 2 == 1) {
        memcpy(a->data, b->data, N * M * sizeof(double));
    }
}