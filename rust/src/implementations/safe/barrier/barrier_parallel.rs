// lib.rs (または main.rs)
use std::thread;
use std::sync::{Arc, Barrier, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Rust: 安全な並列実装 (Mutex/Arc/Barrierを使用)
  境界行のデータ交換にMutex+Copyを使用することで安全性を確保。
  オーバーヘッドは大きくなる可能性があるが、データ競合は確実に回避される。
*/

pub fn barrier_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    let barrier = Arc::new(Barrier::new(2));

    // 境界データの交換用バッファ (ゴーストセル)
    let boundary_mid_minus_1 = Arc::new(Mutex::new(vec![0.0; M])); 
    let boundary_mid = Arc::new(Mutex::new(vec![0.0; M]));

    // aとbそれぞれのデータを上下に分割
    let (a_upper, a_lower) = a.data.split_at_mut(mid * M);
    let (b_upper, b_lower) = b.data.split_at_mut(mid * M);

    thread::scope(|scope| {
        // --- スレッド1: 上半分 (Rows 0 to mid-1) ---
        let barrier1 = barrier.clone();
        let bound_write = boundary_mid_minus_1.clone(); // 自分が書く (最終行 mid-1)
        let bound_read = boundary_mid.clone();          // 相手から読む (0行目 mid)
        
        scope.spawn(move || {
            let mut src = a_upper;
            let mut dst = b_upper;
            let rows = mid; // ローカルな行数

            for _step in 0..steps {
                // 1. 自分の境界行（一番下の行: rows-1）を共有バッファに書き出す
                {
                    let mut writer = bound_write.lock().unwrap();
                    let row_idx = rows - 1;
                    // srcスライスの該当行をコピー
                    writer.copy_from_slice(&src[row_idx * M..(row_idx + 1) * M]);
                }

                // バリア: 両者が境界を書き込むのを待つ
                barrier1.wait();

                // 2. 内部領域の計算 (行 1 .. rows-2)
                for i in 1..rows - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx - M] + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 3. 境界行 (rows-1) の計算
                // 自分の下(idx+M)は相手のバッファ(bound_read)から読み込む
                {
                    let reader = bound_read.lock().unwrap(); // 下半分の0行目(global mid行目)を取得
                    let i = rows - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let down_val = reader[j]; // 共有バッファから読み取り
                        
                        let laplacian = src[idx - M] + down_val + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }
                
                // 固定熱源
                let mid_global_row = N / 2;
                if mid_global_row < mid { // 上半分にある場合
                    dst[mid_global_row * M + M/2] = 100.0;
                }

                // バリア: 計算完了待ち
                barrier1.wait();

                // 参照の入れ替え
                std::mem::swap(&mut src, &mut dst);
            }
        });

        // --- スレッド2: 下半分 (Rows mid to N-1) ---
        let barrier2 = barrier.clone();
        let bound_write = boundary_mid.clone();          // 自分が書く (0行目 mid)
        let bound_read = boundary_mid_minus_1.clone();   // 相手から読む (最終行 mid-1)

        scope.spawn(move || {
            let mut src = a_lower;
            let mut dst = b_lower;
            let rows = N - mid; // ローカルな行数

            for _step in 0..steps {
                // 1. 自分の境界行（一番上の行: 0）を共有バッファに書き出す
                {
                    let mut writer = bound_write.lock().unwrap();
                    // srcスライスの0行目をコピー
                    writer.copy_from_slice(&src[0..M]);
                }

                barrier2.wait();

                // 2. 内部領域の計算 (行 1 .. rows-2)
                for i in 1..rows - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx - M] + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 3. 境界行 (0) の計算
                // 自分の上(idx-M)は相手のバッファ(bound_read)から読み込む
                {
                    let reader = bound_read.lock().unwrap(); // 上半分の最終行を取得
                    let i = 0;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let up_val = reader[j];

                        let laplacian = up_val + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 固定熱源 (下半分にある場合 - 相対座標に変換)
                let mid_global_row = N / 2;
                if mid_global_row >= mid {
                    let local_row = mid_global_row - mid;
                    dst[local_row * M + M/2] = 100.0;
                }

                barrier2.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });
    });

    if steps % 2 == 1 {
        a.data.copy_from_slice(&b.data);
    }
}