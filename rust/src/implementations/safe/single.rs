use crate::grid::{Grid, ALPHA, DT, DX, N, M};

pub fn jacobi_step(current: &Grid, next: &mut Grid){
    let factor = ALPHA * DT / (DX * DX);

    for i in 1..N-1{
        for j in 1..M-1{
            let idx = i * M + j;
            let laplacian = current.data[(i+1) * M + j] + current.data[(i-1) * M + j]
                            + current.data[i * M + (j+1)] + current.data[i * M + (j-1)]
                            - 4.0 * current.data[idx];
            next.data[idx] = current.data[idx] + factor * laplacian;
        }
    }

    // 熱源位置を固定温度に設定（最後に1回だけ）
    next.data[(N/2) * M + M/2] = 100.0;
}
