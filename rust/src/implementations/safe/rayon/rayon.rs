use rayon::prelude::*;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

//書き込み先を完全に分離することで、ロック不要の並列化を実現する
pub fn rayon_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    let mut src = &mut a.data[..];
    let mut dst = &mut b.data[..];

    for _step in 0..steps {
        // 境界行（最上行・最下行）は変化しない（Neumann境界条件）
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        // 内部領域（1行目 ～ N-2行目）のみをスライスとして切り出す。この部分が並列計算の対象
        let interior_dst = &mut dst[M..(N - 1) * M];

        
        //書き込み先のグリッドを「行」単位で分割し、複数のCPUコア（スレッド）に分配して同時に計算
        //各スレッドは異なる行（dst_row）に書き込むため、ロック（Mutexなど）を使わずに安全かつ高速に並列処理が可能です。
        interior_dst
            .par_chunks_mut(M) // 行ごとにスライスを分割
            .enumerate() //各行ごとにインデックスを付与
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

        std::mem::swap(&mut src, &mut dst);
    }

    if steps % 2 != 0 {
        a.data.copy_from_slice(&b.data);
    }
}