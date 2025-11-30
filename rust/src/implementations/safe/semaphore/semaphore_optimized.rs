use core::time;
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
  4. Atomicカウンターで同期（セマフォパターン）

  同期パターン：
  - 初期の境界データを設定
  - 境界データを共有バッファに書き込み
  - カウンターで書き込み完了を通知
  - 相手スレッドの書き込み完了を待つ
  - 全行を計算
  - カウンターで計算完了を通知
  - 相手スレッドの計算完了を待ってからswap
*/

pub fn semaphore_optimized(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    let upper_boundary = Arc::new(Mutex::new(vec![0.0; M]));
    let lower_boundary = Arc::new(Mutex::new(vec![0.0; M]));

    // 初期境界データ設定
    {
        let mut upper_bound = upper_boundary.lock().unwrap();
        let mut lower_bound = lower_boundary.lock().unwrap();
        for j in 0..M {
            upper_bound[j] = grid_a.data[(mid - 1) * M + j];
            lower_bound[j] = grid_a.data[mid * M + j];
        }
    }

    // カウンターは計算完了のみ（これだけで境界データ書き込みも保証される）
    let upper_counter = Arc::new(AtomicUsize::new(0));
    let lower_counter = Arc::new(AtomicUsize::new(0));

    let upper_a: Vec<f64> = grid_a.data[0..mid * M].to_vec();
    let upper_b: Vec<f64> = grid_b.data[0..mid * M].to_vec();
    let lower_a: Vec<f64> = grid_a.data[mid * M..N * M].to_vec();
    let lower_b: Vec<f64> = grid_b.data[mid * M..N * M].to_vec();

    thread::scope(|scope| {
        let upper_cnt = upper_counter.clone();
        let lower_cnt = lower_counter.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let upper_handle = scope.spawn(move || {
            let mut src = upper_a;
            let mut dst = upper_b;

            for step in 0..steps {
                // Thread2の前ステップ計算完了を待つ（step > 0の場合のみ）
                if step > 0 {
                    while lower_cnt.load(Ordering::Acquire) < step {
                        std::hint::spin_loop();
                    }
                }

                // 境界データ書き込み（相手が前ステップを完了しているので安全）
                {
                    let mut upper_bound_row = upper_bound.lock().unwrap();
                    for j in 0..M {
                        upper_bound_row[j] = src[(mid - 1) * M + j];
                    }
                }

                // 内部行計算
                for i in 1..mid.saturating_sub(1) {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 境界行計算
                if mid >= 1 {
                    let lower_bound_row = lower_bound.lock().unwrap();
                    let i = mid - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = lower_bound_row[j] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                if N / 2 < mid {
                    dst[(N / 2) * M + M / 2] = 100.0;
                }

                // 計算完了を通知
                upper_cnt.store(step + 1, Ordering::Release);

                std::mem::swap(&mut src, &mut dst);
            }

            if steps.is_multiple_of(2) { src } else { dst }
        });

        let lower_handle = scope.spawn(move || {
            let mut src = lower_a;
            let mut dst = lower_b;
            let lower_n = N - mid;

            for step in 0..steps {
                if step > 0 {
                    while upper_counter.load(Ordering::Acquire) < step {
                        std::hint::spin_loop();
                    }
                }

                {
                    let mut lower_bound_row = lower_boundary.lock().unwrap();
                    for j in 0..M {
                        lower_bound_row[j] = src[j];
                    }
                }

                for i in 1..lower_n - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                {
                    let upper_bound_row = upper_boundary.lock().unwrap();
                    let i = 0;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + upper_bound_row[j]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                if N / 2 >= mid {
                    let heat_i = N / 2 - mid;
                    dst[heat_i * M + M / 2] = 100.0;
                }

                lower_counter.store(step + 1, Ordering::Release);

                std::mem::swap(&mut src, &mut dst);
            }

            if steps.is_multiple_of(2) { src } else { dst }
        });

        let final_upper = upper_handle.join().unwrap();
        let final_lower = lower_handle.join().unwrap();

        grid_a.data[0..mid * M].copy_from_slice(&final_upper);
        grid_a.data[mid * M..N * M].copy_from_slice(&final_lower);
    });
}