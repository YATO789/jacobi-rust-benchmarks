use crate::grid::{Grid, ALPHA, DT, DX, N, M};

pub fn jacobi_step(current: &Grid, next: &mut Grid){
    for i in 1..N-1{
        for j in 1..M-1{
            // 熱源位置は固定温度として扱う
            if i == N/2 && j == M/2 {
                next.data[i * M + j] = 100.0;
                continue;
            }
            let idx = i * M + j;
            let laplacian = current.data[(i+1) * M + j] + current.data[(i-1) * M + j]
                            + current.data[i * M + (j+1)] + current.data[i * M + (j-1)]
                            - 4.0 * current.data[idx];
            next.data[idx] = current.data[idx] + ALPHA * DT / (DX*DX) * laplacian;
        }
    }
}
