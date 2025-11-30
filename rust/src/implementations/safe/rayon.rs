use rayon::prelude::*;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Rayon ベースの並列実装

  主な特徴：
  1. Rayonのwork-stealingスレッドプールを使用した自動負荷分散
  2. par_iterとcollectを使った安全な並列イテレーション
  3. 各行を独立して並列計算
  4. 手動のスレッド管理や同期が不要
*/

pub fn rayon_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    for step in 0..steps {
        let (src, dst) = if step.is_multiple_of(2) {
            (&a.data[..], &mut b.data[..])
        } else {
            (&b.data[..], &mut a.data[..])
        };

        // 境界行をコピー
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        // 行レベルの並列処理で内部点を計算
        let results: Vec<Vec<f64>> = (1..N - 1)
            .into_par_iter()
            .map(|i| {
                let mut row_result = vec![0.0; M];

                // 境界は変更なし
                row_result[0] = src[i * M];
                row_result[M - 1] = src[i * M + M - 1];

                // 内部点を計算
                for j in 1..M - 1 {
                    let idx = i * M + j;
                    let laplacian = src[idx + M]
                        + src[idx - M]
                        + src[idx + 1]
                        + src[idx - 1]
                        - 4.0 * src[idx];

                    row_result[j] = src[idx] + factor * laplacian;
                }

                row_result
            })
            .collect();

        // 計算結果を書き戻し
        for (row_idx, row) in results.iter().enumerate() {
            let i = row_idx + 1;
            dst[i * M..(i + 1) * M].copy_from_slice(row);
        }

        // 熱源を固定温度に設定
        dst[(N / 2) * M + M / 2] = 100.0;
    }

    // 最終結果がgrid aにあることを保証
    if !steps.is_multiple_of(2) {
        a.data.copy_from_slice(&b.data);
    }
}

/*
  par_chunks_mutを使った行レベル並列処理の最適化版
  最も効率的なsafe実装
*/
pub fn rayon_parallel_v2(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    for step in 0..steps {
        let (src, dst) = if step.is_multiple_of(2) {
            (&a.data[..], &mut b.data[..])
        } else {
            (&b.data[..], &mut a.data[..])
        };

        // 最初に境界をコピー
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        // 内部行を並列処理
        // 並列での可変アクセスを許可するために、宛先スライスを分割する必要がある
        let interior_start = M;
        let interior_end = (N - 1) * M;

        dst[interior_start..interior_end]
            .par_chunks_mut(M)
            .enumerate()
            .for_each(|(row_idx, dst_row)| {
                let i = row_idx + 1; // グリッド内の実際の行インデックス

                for j in 1..M - 1 {
                    let idx = i * M + j;
                    let laplacian = src[idx + M]
                        + src[idx - M]
                        + src[idx + 1]
                        + src[idx - 1]
                        - 4.0 * src[idx];

                    dst_row[j] = src[idx] + factor * laplacian;
                }

                // この行の境界値をコピー
                dst_row[0] = src[i * M];
                dst_row[M - 1] = src[i * M + M - 1];
            });

        // 熱源を固定温度に設定
        dst[(N / 2) * M + M / 2] = 100.0;
    }

    // 最終結果がgrid aにあることを保証
    if !steps.is_multiple_of(2) {
        a.data.copy_from_slice(&b.data);
    }
}