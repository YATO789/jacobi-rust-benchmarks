use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  最適化されたsafe semaphore実装

  主な最適化：
  1. グリッド全体ではなく、データを上下に分割して各スレッドが所有
  2. 境界行のみをMutex<Vec<f64>>で共有（最小限のロック範囲）
  3. Grid全体のcloneを排除
  4. Atomicカウンターで同期（セマフォパターン、1回のみ）

  同期パターン（barrier_parallel_03と同じ）：
  - 初期の境界データを設定
  - 前ステップの境界データを使って全行を計算
  - カウンターで計算完了を通知
  - 相手スレッドの計算完了を待ってからswap
  - swap後、次ステップ用の境界データを更新
*/

pub fn semaphore_optimized(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // 境界行の共有バッファ
    let upper_boundary = Arc::new(Mutex::new(vec![0.0; M])); // mid-1行目
    let lower_boundary = Arc::new(Mutex::new(vec![0.0; M])); // mid行目

    // 初期の境界データを設定
    {
        let mut upper_bound = upper_boundary.lock().unwrap();
        let mut lower_bound = lower_boundary.lock().unwrap();
        for j in 0..M {
            upper_bound[j] = grid_a.data[(mid - 1) * M + j];
            lower_bound[j] = grid_a.data[mid * M + j];
        }
    }

    // カウンターで同期（計算完了を通知）
    let upper_counter = Arc::new(AtomicUsize::new(0));
    let lower_counter = Arc::new(AtomicUsize::new(0));

    // データを分割（各スレッドが自分の領域を所有）
    let upper_a: Vec<f64> = grid_a.data[0..mid * M].to_vec();
    let upper_b: Vec<f64> = grid_b.data[0..mid * M].to_vec();
    let lower_a: Vec<f64> = grid_a.data[mid * M..N * M].to_vec();
    let lower_b: Vec<f64> = grid_b.data[mid * M..N * M].to_vec();

    thread::scope(|scope| {
        // スレッド1: 上半分を処理
        let upper_counter_clone = upper_counter.clone();
        let lower_counter_clone = lower_counter.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let upper_handle = scope.spawn(move || {
            let mut src = upper_a;
            let mut dst = upper_b;

            for step in 0..steps {
                // 内部行を計算（ロックフリー）
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

                // 計算完了を通知
                upper_counter_clone.store(step + 1, Ordering::Release);

                // Thread2の計算完了を待つ
                while lower_counter_clone.load(Ordering::Acquire) < step + 1 {
                    std::hint::spin_loop();
                }

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

        // スレッド2: 下半分を処理
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let lower_handle = scope.spawn(move || {
            let mut src = lower_a;
            let mut dst = lower_b;
            let lower_n = N - mid;

            for step in 0..steps {
                // 内部行を計算（ロックフリー）
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

                // 計算完了を通知
                lower_counter.store(step + 1, Ordering::Release);

                // Thread1の計算完了を待つ
                while upper_counter.load(Ordering::Acquire) < step + 1 {
                    std::hint::spin_loop();
                }

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
        grid_a.data[0..mid * M].copy_from_slice(&final_upper);
        grid_a.data[mid * M..N * M].copy_from_slice(&final_lower);
    });
}
