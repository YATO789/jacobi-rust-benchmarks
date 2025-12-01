use rayon::prelude::*;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Rayon最適化版 (v3 - Fixed)
  
  修正点:
  1. with_min_len の設定値を「行数」に修正
  2. 行ごとのスライス参照による境界チェック最適化は維持
*/
pub fn rayon_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);
    
    // スレッド数に応じて、1タスクあたりの最小行数を計算
    // 例: 200行 / (2スレッド * 4分割) = 25行
    let num_threads = rayon::current_num_threads();
    let min_rows_per_task = N / (num_threads * 4);

    for step in 0..steps {
        let (src, dst) = if step % 2 == 0 {
            (&a.data[..], &mut b.data[..])
        } else {
            (&b.data[..], &mut a.data[..])
        };

        // 境界行コピー
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        let interior_dst = &mut dst[M..(N - 1) * M];

        interior_dst
            .par_chunks_mut(M)
            .with_min_len(min_rows_per_task) // 【修正】 * M を削除
            .enumerate()
            .for_each(|(r, dst_row)| {
                // rは内部領域での行インデックス (0始まり)
                // 実際のgrid上の行は r + 1
                let i = r + 1;

                // 3本のポインタ（スライス）を取り出すことで
                // コンパイラにメモリ配置の連続性をヒントとして与える
                let src_up = &src[(i - 1) * M..i * M];
                let src_mid = &src[i * M..(i + 1) * M];
                let src_down = &src[(i + 1) * M..(i + 2) * M];

                for j in 1..M - 1 {
                    let laplacian = src_up[j]
                        + src_down[j]
                        + src_mid[j - 1]
                        + src_mid[j + 1]
                        - 4.0 * src_mid[j];

                    dst_row[j] = src_mid[j] + factor * laplacian;
                }

                dst_row[0] = src_mid[0];
                dst_row[M - 1] = src_mid[M - 1];
            });

        dst[(N / 2) * M + M / 2] = 100.0;
    }

    if steps % 2 != 0 {
        a.data.copy_from_slice(&b.data);
    }
}