use std::thread;
use std::sync::{Arc, Barrier, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  バリア同期を1回のみ使用する最適化版

  主な改善点：
  1. ステップあたりバリア1回のみ（従来は2回）
  2. 境界データの更新を計算後に行う
  3. ダブルバッファリングを活用して依存関係を解決

  動作フロー（各ステップ）：
  - 前ステップの境界データを使って全行を計算
  - バリア同期（計算完了を待つ）
  - swap + 次ステップ用の境界データを更新
*/

pub fn barrier_parallel_03(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // 境界行の共有バッファ（2つのバッファを持つ）
    let upper_boundary = Arc::new(Mutex::new(vec![0.0; M])); // mid-1行目
    let lower_boundary = Arc::new(Mutex::new(vec![0.0; M])); // mid行目

    let barrier = Arc::new(Barrier::new(2));

    // 初期の境界データを設定
    {
        let mut upper_bound = upper_boundary.lock().unwrap();
        let mut lower_bound = lower_boundary.lock().unwrap();
        for j in 0..M {
            upper_bound[j] = a.data[(mid - 1) * M + j];
            lower_bound[j] = a.data[mid * M + j];
        }
    }

    // データを分割
    let upper_a: Vec<f64> = a.data[0..mid * M].to_vec();
    let upper_b: Vec<f64> = b.data[0..mid * M].to_vec();
    let lower_a: Vec<f64> = a.data[mid * M..N * M].to_vec();
    let lower_b: Vec<f64> = b.data[mid * M..N * M].to_vec();

    thread::scope(|scope| {
        // スレッド1: 上半分を処理 (0..mid)
        let barrier1 = barrier.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let upper_handle = scope.spawn(move || {
            let mut src = upper_a;
            let mut dst = upper_b;

            for _step in 0..steps {
                // 内部行を計算 (1..mid-1, ロックフリー)
                for i in 1..mid.saturating_sub(1) {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M]
                            + src[idx - M]
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 境界行（mid-1行目）を計算（共有バッファから読み取り）
                if mid >= 1 {
                    let lower_bound_row = lower_bound.lock().unwrap();
                    let i = mid - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = lower_bound_row[j]  // 共有バッファから
                            + src[idx - M]
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 熱源位置を固定温度に設定（上半分に含まれる場合）
                if N / 2 < mid {
                    dst[(N / 2) * M + M / 2] = 100.0;
                }

                // バリア: 全計算完了を待つ
                barrier1.wait();

                // swap
                std::mem::swap(&mut src, &mut dst);

                // 次のステップ用に境界データを更新
                {
                    let mut upper_bound_row = upper_bound.lock().unwrap();
                    for j in 0..M {
                        upper_bound_row[j] = src[(mid - 1) * M + j];
                    }
                }
            }

            if steps.is_multiple_of(2) { src } else { dst }
        });

        // スレッド2: 下半分を処理 (mid..N)
        let barrier2 = barrier.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let lower_handle = scope.spawn(move || {
            let mut src = lower_a;
            let mut dst = lower_b;
            let lower_n = N - mid;

            for _step in 0..steps {
                // 内部行を計算 (1..lower_n-1, ロックフリー)
                for i in 1..lower_n - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M]
                            + src[idx - M]
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 境界行（0 = mid行目）を計算（共有バッファから読み取り）
                {
                    let upper_bound_row = upper_bound.lock().unwrap();
                    let i = 0;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M]
                            + upper_bound_row[j]  // 共有バッファから
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 熱源位置を固定温度に設定（下半分に含まれる場合）
                if N / 2 >= mid {
                    let heat_i = N / 2 - mid;
                    dst[heat_i * M + M / 2] = 100.0;
                }

                // バリア: 全計算完了を待つ
                barrier2.wait();

                // swap
                std::mem::swap(&mut src, &mut dst);

                // 次のステップ用に境界データを更新
                {
                    let mut lower_bound_row = lower_bound.lock().unwrap();
                    for j in 0..M {
                        lower_bound_row[j] = src[j];
                    }
                }
            }

            if steps.is_multiple_of(2) { src } else { dst }
        });

        // 結果を統合
        let final_upper = upper_handle.join().unwrap();
        let final_lower = lower_handle.join().unwrap();

        // 元のグリッドに書き戻し
        a.data[0..mid * M].copy_from_slice(&final_upper);
        a.data[mid * M..N * M].copy_from_slice(&final_lower);
    });
}
