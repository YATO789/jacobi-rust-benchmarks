use std::thread;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

// std::thread版: 2スレッド並列
pub fn jacobi_step_parallel(current: &Grid, next: &mut Grid) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);


    thread::scope(|scope|{
        //mut upper_rows  mut lower_rows
        let (upper_rows, lower_rows) = next.data.split_at_mut(mid);
        let current_data = &current.data;

    });
}