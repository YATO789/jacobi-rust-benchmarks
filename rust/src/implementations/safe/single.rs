use std::mem;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

pub fn jacobi_step(a: &mut Grid, b: &mut Grid,steps:usize){
    let factor = ALPHA * DT / (DX * DX);

    for _ in 0..steps{

        for i in 1..N-1{
            for j in 1..M-1{
                let idx = i * M + j;
                let laplacian = a.data[(i+1) * M + j] + a.data[(i-1) * M + j]
                                + a.data[i * M + (j+1)] + a.data[i * M + (j-1)]
                                - 4.0 * a.data[idx];
                b.data[idx] = a.data[idx] + factor * laplacian;
            }
        }

        // 熱源位置を固定温度に設定（最後に1回だけ）
        b.data[(N/2) * M + M/2] = 100.0;

        mem::swap(&mut a.data, &mut b.data);
    }
}
