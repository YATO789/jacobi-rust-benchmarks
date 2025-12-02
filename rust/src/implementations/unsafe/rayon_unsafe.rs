use rayon::prelude::*;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Rayon Unsafe版

  Safe版との違い:
  - get_unchecked を使用して境界チェックを回避
  - 配列アクセスの境界チェックオーバーヘッドを削減

  注意:
  - インデックスの範囲は論理的に保証されているため安全
  - ベンチマーク研究用の実装
*/
pub fn rayon_unsafe(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    // スレッド数に応じて、1タスクあたりの最小行数を計算
    let num_threads = rayon::current_num_threads();
    let min_rows_per_task = N / (num_threads * 4);

    for step in 0..steps {
        let (src, dst) = if step % 2 == 0 {
            (&a.data[..], &mut b.data[..])
        } else {
            (&b.data[..], &mut a.data[..])
        };

        // 境界行コピー（Safe版と同じ）
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        let interior_dst = &mut dst[M..(N - 1) * M];

        interior_dst
            .par_chunks_mut(M)
            .with_min_len(min_rows_per_task)
            .enumerate()
            .for_each(|(r, dst_row)| {
                // rは内部領域での行インデックス (0始まり)
                // 実際のgrid上の行は r + 1
                let i = r + 1;

                // SAFETY: インデックスは常に有効な範囲内
                // - i >= 1 かつ i < N-1 が保証されている
                // - 各行のサイズはM
                unsafe {
                    let src_up_start = (i - 1) * M;
                    let src_mid_start = i * M;
                    let src_down_start = (i + 1) * M;

                    for j in 1..M - 1 {
                        // get_unchecked で境界チェックを回避
                        let v = *src.get_unchecked(src_mid_start + j);
                        let laplacian = *src.get_unchecked(src_up_start + j)
                            + *src.get_unchecked(src_down_start + j)
                            + *src.get_unchecked(src_mid_start + j - 1)
                            + *src.get_unchecked(src_mid_start + j + 1)
                            - 4.0 * v;

                        *dst_row.get_unchecked_mut(j) = v + factor * laplacian;
                    }

                    // 境界列のコピー
                    *dst_row.get_unchecked_mut(0) = *src.get_unchecked(src_mid_start);
                    *dst_row.get_unchecked_mut(M - 1) = *src.get_unchecked(src_mid_start + M - 1);
                }
            });

        // 固定熱源
        dst[(N / 2) * M + M / 2] = 100.0;
    }

    // 結果を a に戻す
    if steps % 2 != 0 {
        a.data.copy_from_slice(&b.data);
    }
}
