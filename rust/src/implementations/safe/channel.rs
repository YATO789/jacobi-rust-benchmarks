use std::thread;
use std::sync::{Arc, Barrier,mpsc};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  unsafeを使わない最適化版並列実装

  主な最適化：
  1. データを上半分と下半分に分割して独立処理
  2. 境界行のみをMutexで共有（ロック範囲を最小化）
  3. 大部分の計算はロックフリー
  4. バリア同期でステップ完了を保証
  5. メモリコピーを最小限に抑える
*/

pub fn channel_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // バッファ付きチャネル（デッドロック回避）
    let (tx1, rx1) = mpsc::sync_channel::<Vec<f64>>(1);
    let (tx2, rx2) = mpsc::sync_channel::<Vec<f64>>(1);

    let barrier = Arc::new(Barrier::new(2));

    let upper_a: Vec<f64> = a.data[0..mid * M].to_vec();
    let upper_b: Vec<f64> = b.data[0..mid * M].to_vec();
    let lower_a: Vec<f64> = a.data[mid * M..N * M].to_vec();
    let lower_b: Vec<f64> = b.data[mid * M..N * M].to_vec();

    thread::scope(|scope| {
        let barrier1 = barrier.clone();

        let upper_handle = scope.spawn(move || {
            let mut src = upper_a;
            let mut dst = upper_b;
            let mut upper_bound_row = vec![0.0; M]; // ループ外で1回だけ割り当て

            for _step in 0..steps {
                // 境界データを準備して送信
                for j in 0..M {
                    upper_bound_row[j] = src[(mid - 1) * M + j];
                }
                tx1.send(upper_bound_row.clone())
                    .expect("Failed to send upper boundary");

                // 相手の境界データを受信
                let lower_bound = rx2.recv()
                    .expect("Failed to receive lower boundary");

                // 内部行を計算
                for i in 1..mid.saturating_sub(1) {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 境界行を計算
                if mid >= 1 {
                    let i = mid - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = lower_bound[j] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                if N / 2 < mid {
                    dst[(N / 2) * M + M / 2] = 100.0;
                }

                // 計算完了を待ってswap
                barrier1.wait();
                std::mem::swap(&mut src, &mut dst);
            }

            if steps.is_multiple_of(2) { src } else { dst }
        });

        let barrier2 = barrier.clone();

        let lower_handle = scope.spawn(move || {
            let mut src = lower_a;
            let mut dst = lower_b;
            let lower_n = N - mid;
            let mut lower_bound_row = vec![0.0; M];

            for _step in 0..steps {
                for j in 0..M {
                    lower_bound_row[j] = src[j];
                }
                tx2.send(lower_bound_row.clone())
                    .expect("Failed to send lower boundary");

                let upper_bound = rx1.recv()
                    .expect("Failed to receive upper boundary");

                for i in 1..lower_n - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                let i = 0;
                for j in 1..M - 1 {
                    let idx = i * M + j;
                    let laplacian = src[idx + M] + upper_bound[j]
                        + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                    dst[idx] = src[idx] + factor * laplacian;
                }

                if N / 2 >= mid {
                    let heat_i = N / 2 - mid;
                    dst[heat_i * M + M / 2] = 100.0;
                }

                barrier2.wait();
                std::mem::swap(&mut src, &mut dst);
            }

            if steps.is_multiple_of(2) { src } else { dst }
        });

        let final_upper = upper_handle.join().unwrap();
        let final_lower = lower_handle.join().unwrap();

        a.data[0..mid * M].copy_from_slice(&final_upper);
        a.data[mid * M..N * M].copy_from_slice(&final_lower);
    });
}