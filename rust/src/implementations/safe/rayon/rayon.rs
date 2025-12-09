use rayon::prelude::*;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

//書き込み先を完全に分離することで、ロック不要の並列化を実現する
pub fn rayon_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    //ここでは src（読み取り元）と dst（書き込み先）という2つのスライスを用意
    let mut src = &mut a.data[..];
    let mut dst = &mut b.data[..];

    for _step in 0..steps {
        // 境界行（最上行・最下行）は変化しない（Neumann境界条件）
        //「計算できない端っこの値を、最新のバッファ（dst）にも正しく引き継ぐ」ために不可欠な処理
        dst[0..M].copy_from_slice(&src[0..M]); //一番上の行
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]); //一番下の行

        // 内部領域（1行目 ～ N-2行目）のみをスライスとして切り出す。この部分が並列計算の対象
        let interior_dst = &mut dst[M..(N - 1) * M];

        
        //書き込み先のグリッドを「行」単位で分割し、複数のCPUコア（スレッド）に分配して同時に計算
        //各スレッドは異なる行（dst_row）に書き込むため、ロック（Mutexなど）を使わずに安全かつ高速に並列処理が可能です。
        interior_dst
            .par_chunks_mut(M) // 行ごとにスライスを分割
            .enumerate() //各行ごとにインデックスを付与
            .for_each(|(r, dst_row)| {
                //以下各スレッドで実行

                // rは切り出した内部領域の中での行番号 (0始まり)
                // 実際のgrid上の行は r + 1
                let i = r + 1;

                for j in 1..M - 1 {
                    let idx = i * M + j;
                    let laplacian = src[idx - M] + src[idx + M] +
                                    src[idx - 1] + src[idx + 1] -
                                    4.0 * src[idx];
                    dst_row[j] = src[idx] + factor * laplacian;
                }

                dst_row[0] = src[i * M];
                dst_row[M - 1] = src[i * M + M - 1];
            });

        dst[(N / 2) * M + M / 2] = 100.0;

        std::mem::swap(&mut src, &mut dst);
    }

    if steps % 2 != 0 {
        a.data.copy_from_slice(&b.data);
    }
}