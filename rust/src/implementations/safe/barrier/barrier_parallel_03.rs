use std::thread;
use std::sync::{Arc, Barrier, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  バリア同期版（barrier_parallel_02と同様の実装）

  主な特徴：
  1. データを上半分と下半分に分割して独立処理
  2. 境界行のみをMutexで共有（ロック範囲を最小化）
  3. 大部分の計算はロックフリー
  4. バリア同期でステップ完了を保証

  動作フロー（各ステップ）：
  - 内部行と境界行を計算
  - バリア同期（計算完了を待つ）
  - 次ステップ用の境界データを更新
  - swap

  注：当初「バリア1回のみ」を目指したが、正確性のため barrier_parallel_02 と同等の実装になった
*/

pub fn barrier_parallel_03(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // 境界行の共有バッファ
    let upper_boundary = Arc::new(Mutex::new(vec![0.0; M])); // mid-1行目
    let lower_boundary = Arc::new(Mutex::new(vec![0.0; M])); // mid行目

    let barrier = Arc::new(Barrier::new(2));

    // データを分割
    //元の a.dataとb.dataとは完全に独立した別のメモリ領域
    let upper_a: Vec<f64> = a.data[0..mid * M].to_vec();
    let upper_b: Vec<f64> = b.data[0..mid * M].to_vec();
    let lower_a: Vec<f64> = a.data[mid * M..N * M].to_vec();
    let lower_b: Vec<f64> = b.data[mid * M..N * M].to_vec();

    // 初期の境界データを設定
    {
        let mut upper_bound = upper_boundary.lock().unwrap();
        let mut lower_bound = lower_boundary.lock().unwrap();
        for j in 0..M {
            upper_bound[j] = a.data[(mid - 1) * M + j];
            lower_bound[j] = a.data[mid * M + j];
        }
    }

    thread::scope(|scope| {
        // スレッド1: 上半分を処理 (0..mid)
        let barrier1 = barrier.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let upper_handle = scope.spawn(move || {
            let mut src = upper_a;
            let mut dst = upper_b;

            for _step in 0..steps {
                // 次のステップ用に境界データを更新（計算前に！）
                {
                    let mut upper_bound_row = upper_bound.lock().unwrap();
                    for j in 0..M {
                        upper_bound_row[j] = src[(mid - 1) * M + j];
                    }
                }

                // バリア: 両スレッドが境界行を書き込むまで待機
                barrier1.wait();

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

                // バリア: 全ての計算が完了するまで待機してから入れ替え
                barrier1.wait();

                // swap
                std::mem::swap(&mut src, &mut dst);
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
                // 次のステップ用に境界データを更新（計算前に！）
                {
                    let mut lower_bound_row = lower_bound.lock().unwrap();
                    for j in 0..M {
                        lower_bound_row[j] = src[j];
                    }
                }

                // バリア: 両スレッドが境界行を書き込むまで待機
                barrier2.wait();

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

                // バリア: 全ての計算が完了するまで待機してから入れ替え
                barrier2.wait();

                // swap
                std::mem::swap(&mut src, &mut dst);
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
