use jacobi_rust::grid::{Grid, TIME_STEPS};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter as unsafe_semaphore;
use jacobi_rust::implementations::safe::semaphore::semaphore_optimized::semaphore_optimized;
use jacobi_rust::implementations::safe::rayon::rayon::rayon_parallel;
use jacobi_rust::implementations::safe::channel::channel::channel_parallel;
use jacobi_rust::implementations::unsafe_impl::parallel_unsafe::unsafe_optimized;

fn main() {
    let test_steps = 100; // テスト用のステップ数

    println!("=== Rust実装の結果出力テスト ===");
    println!("ステップ数: {}", test_steps);
    println!();

    // 各実装をテスト
    let tests = vec![
        ("single", run_single as fn(usize) -> Grid),
        ("unsafe_semaphore", run_unsafe_semaphore),
        ("safe_semaphore", run_safe_semaphore),
        ("barrier", run_barrier),
        ("rayon", run_rayon),
        ("channel", run_channel),
        ("unsafe_parallel", run_unsafe_parallel),
    ];

    for (name, test_fn) in tests {
        let result = test_fn(test_steps);
        let filename = format!("rust_{}.bin", name);

        result.save_to_file(&filename).expect("Failed to save file");
        println!("✓ {} -> {}", name, filename);

        // 中心点と周辺の値を表示（デバッグ用）
        let n = jacobi_rust::grid::N;
        let m = jacobi_rust::grid::M;
        let center_idx = (n / 2) * m + (m / 2);
        println!("  中心点 [{}][{}] = {:.6}", n/2, m/2, result.data[center_idx]);

        // 4隅の値を表示
        println!("  左上 [0][0] = {:.6}", result.data[0]);
        println!("  右上 [0][{}] = {:.6}", m-1, result.data[m-1]);
        println!("  左下 [{}][0] = {:.6}", n-1, result.data[(n-1)*m]);
        println!("  右下 [{}][{}] = {:.6}", n-1, m-1, result.data[(n-1)*m + m-1]);
        println!();
    }

    println!("全ての結果ファイルを出力しました。");
}

fn run_single(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    jacobi_step(&mut a, &mut b, steps);
    a
}

fn run_unsafe_semaphore(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    unsafe_semaphore(&mut a, &mut b, steps);
    a
}

fn run_safe_semaphore(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    semaphore_optimized(&mut a, &mut b, steps);
    a
}

fn run_barrier(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    barrier_parallel(&mut a, &mut b, steps);
    a
}

fn run_rayon(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    rayon_parallel(&mut a, &mut b, steps);
    a
}

fn run_channel(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    channel_parallel(&mut a, &mut b, steps);
    a
}

fn run_unsafe_parallel(steps: usize) -> Grid {
    let mut a = Grid::new();
    let mut b = Grid::new();
    unsafe_optimized(&mut a, &mut b, steps);
    a
}
