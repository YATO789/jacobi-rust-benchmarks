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

  ロジック:
  - Safe版と同じダブルバッファリング方式（std::mem::swap使用）
  - 書き込み先を完全に分離することで、ロック不要の並列化を実現
*/
pub fn rayon_unsafe(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    let mut src = &mut a.data[..];
    let mut dst = &mut b.data[..];

    for _step in 0..steps {
        // 境界行（最上行・最下行）は変化しない（Neumann境界条件）
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        // 内部領域（1行目 ～ N-2行目）のみをスライスとして切り出す。この部分が並列計算の対象
        let interior_dst = &mut dst[M..(N - 1) * M];

        // 書き込み先のグリッドを「行」単位で分割し、複数のCPUコア（スレッド）に分配して同時に計算
        // 各スレッドは異なる行（dst_row）に書き込むため、ロック（Mutexなど）を使わずに安全かつ高速に並列処理が可能
        interior_dst
            .par_chunks_mut(M) // 行ごとにスライスを分割
            .enumerate() // 各行ごとにインデックスを付与
            .for_each(|(r, dst_row)| {
                // rは内部領域での行インデックス (0始まり)
                // 実際のgrid上の行は r + 1
                let i = r + 1;

                // SAFETY: インデックスは常に有効な範囲内
                // - i >= 1 かつ i < N-1 が保証されている
                // - idx の範囲も論理的に保証されている
                unsafe {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        // get_unchecked で境界チェックを回避
                        let laplacian = *src.get_unchecked(idx - M)
                            + *src.get_unchecked(idx + M)
                            + *src.get_unchecked(idx - 1)
                            + *src.get_unchecked(idx + 1)
                            - 4.0 * *src.get_unchecked(idx);

                        *dst_row.get_unchecked_mut(j) = *src.get_unchecked(idx) + factor * laplacian;
                    }

                    // 境界列のコピー
                    *dst_row.get_unchecked_mut(0) = *src.get_unchecked(i * M);
                    *dst_row.get_unchecked_mut(M - 1) = *src.get_unchecked(i * M + M - 1);
                }
            });

        // 固定熱源
        dst[(N / 2) * M + M / 2] = 100.0;

        std::mem::swap(&mut src, &mut dst);
    }

    // 結果を a に戻す
    if steps % 2 != 0 {
        a.data.copy_from_slice(&b.data);
    }
}
