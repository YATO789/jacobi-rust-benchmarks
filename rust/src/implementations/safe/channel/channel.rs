use std::thread;
use std::sync::{Arc, Barrier, mpsc};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Channel版（最適化済み）
  
  改良点:
  1. split_at_mut を使用し、メインメモリのコピーを完全に排除
  2. チャネルによる境界データ交換は維持（アーキテクチャの比較用として）
  3. is_multiple_of を標準的な % 演算子に変更
*/

pub fn channel_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // バッファ付きチャネル
    let (tx1, rx1) = mpsc::sync_channel::<Vec<f64>>(1);
    let (tx2, rx2) = mpsc::sync_channel::<Vec<f64>>(1);

    let barrier = Arc::new(Barrier::new(2));

    // 【重要】コピーではなく、可変スライスとして分割 (Zero-Copy)
    let (a_upper, a_lower) = a.data.split_at_mut(mid * M);
    let (b_upper, b_lower) = b.data.split_at_mut(mid * M);

    thread::scope(|scope| {
        // --- 上半分スレッド ---
        let barrier1 = barrier.clone();
        
        scope.spawn(move || {
            let mut src = a_upper;
            let mut dst = b_upper;
            let rows = mid; // ローカルな行数
            
            // 送信用バッファ（ループ内で使い回すが、cloneでコピーが発生する）
            let mut boundary_buffer = vec![0.0; M];

            for _step in 0..steps {
                // 1. 境界データを準備して送信
                // (mid-1行目 = ローカルの最終行)
                let last_row_idx = rows - 1;
                boundary_buffer.copy_from_slice(&src[last_row_idx * M .. (last_row_idx + 1) * M]);
                
                // clone()で新しいVecを作って送信 (mpscの制約)
                tx1.send(boundary_buffer.clone())
                    .expect("Failed to send upper boundary");

                // 2. 相手の境界データを受信
                let lower_bound = rx2.recv()
                    .expect("Failed to receive lower boundary");

                // 3. 内部計算
                for i in 1..rows - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 4. 境界行の計算
                // 自分の下(i+1)は受信データ(lower_bound)を使う
                {
                    let i = rows - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let down_val = lower_bound[j];
                        let laplacian = down_val + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 固定熱源
                if N / 2 < mid {
                    dst[(N / 2) * M + M / 2] = 100.0;
                }

                // 5. バリア同期（計算完了待ち）
                barrier1.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });

        // --- 下半分スレッド ---
        let barrier2 = barrier.clone();

        scope.spawn(move || {
            let mut src = a_lower;
            let mut dst = b_lower;
            let rows = N - mid;
            
            let mut boundary_buffer = vec![0.0; M];

            for _step in 0..steps {
                // 1. 境界データを準備して送信
                // (0行目)
                boundary_buffer.copy_from_slice(&src[0..M]);
                
                tx2.send(boundary_buffer.clone())
                    .expect("Failed to send lower boundary");

                // 2. 相手の境界データを受信
                let upper_bound = rx1.recv()
                    .expect("Failed to receive upper boundary");

                // 3. 内部計算
                for i in 1..rows - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M] + src[idx - M]
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 4. 境界行の計算
                // 自分の上(i-1)は受信データ(upper_bound)を使う
                {
                    let i = 0;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let up_val = upper_bound[j];
                        let laplacian = src[idx + M] + up_val
                            + src[idx + 1] + src[idx - 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 固定熱源
                if N / 2 >= mid {
                    let local_heat_row = N / 2 - mid;
                    dst[local_heat_row * M + M / 2] = 100.0;
                }

                // 5. バリア同期
                barrier2.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });
    });

    // 奇数ステップ時の書き戻し（a_upperなどは可変参照なので、直接書き変わっている）
    if steps % 2 == 1 {
        a.data.copy_from_slice(&b.data);
    }
}