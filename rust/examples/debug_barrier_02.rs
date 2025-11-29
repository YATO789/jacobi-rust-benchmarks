use jacobi_rust::grid::{Grid, N, M};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_02::barrier_parallel_02;

fn main() {
    println!("N={}, M={}, mid={}", N, M, N/2);

    // シングル版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..10 {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }
    let single_final = &single_a;

    // 並列版
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_parallel_02(&mut grid_a, &mut grid_b, 10);

    let parallel_final = &grid_b;

    println!("\nComparing single vs parallel:");
    for i in vec![1, 2, 490, 498, 499, 500, 501, 997, 998] {
        let idx = i * M + 500;
        println!("  [{}][500]: single={}, parallel={}",
            i, single_final.data[idx], parallel_final.data[idx]);
    }

    // 熱源を確認
    println!("\nHeat source:");
    println!("  single [{}][{}]: {}", N/2, M/2, single_final.data[N/2 * M + M/2]);
    println!("  parallel [{}][{}]: {}", N/2, M/2, parallel_final.data[N/2 * M + M/2]);
}
