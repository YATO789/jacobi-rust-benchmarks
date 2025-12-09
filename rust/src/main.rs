use std::time::{Duration, Instant};
use jacobi_rust::grid::{Grid, TIME_STEPS, WARMUP_STEPS};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter as unsafe_semaphore;
use jacobi_rust::implementations::safe::semaphore::semaphore_optimized::semaphore_optimized;
use jacobi_rust::implementations::safe::rayon::rayon::rayon_parallel;
use jacobi_rust::implementations::unsafe_impl::barrier_unsafe::barrier_unsafe;
use jacobi_rust::implementations::unsafe_impl::rayon_unsafe::rayon_unsafe;

const BENCH_ITERATIONS: usize = 15;
const BENCH_WARMUP: usize = 3;

fn main() {
    // コマンドライン引数でスレッド数を指定可能
    let args: Vec<String> = std::env::args().collect();
    let num_threads = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or_else(|_| {
            eprintln!("エラー: スレッド数は正の整数である必要があります");
            std::process::exit(1);
        })
    } else {
        2 // デフォルトは2スレッド
    };

    if num_threads < 1 {
        eprintln!("エラー: スレッド数は1以上である必要があります");
        std::process::exit(1);
    }

    // Rayonのスレッド数を設定
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    println!("=== Jacobi法 2D熱方程式ベンチマーク ===");
    println!("TIME_STEPS: {}, 測定回数: {}, スレッド数: {}\n", TIME_STEPS, BENCH_ITERATIONS, num_threads);

    run_benchmark("Single Thread", run_single);
    run_benchmark("Unsafe Semaphore", run_unsafe_semaphore);
    run_benchmark("Safe Semaphore", run_safe_semaphore_optimized);
    run_benchmark("Barrier", run_barrier_parallel_02);
    run_benchmark("Barrier Unsafe", run_barrier_unsafe);
    run_benchmark("Rayon", run_rayon_v2);
    run_benchmark("Rayon Unsafe", run_rayon_unsafe);
    println!("\n=== ベンチマーク完了 ===");
}

fn run_benchmark<F>(name: &str, mut bench_fn: F)
where
    F: FnMut() -> Duration,
{
    println!("{}:", name);

    // ウォームアップ
    for _ in 0..BENCH_WARMUP {
        bench_fn();
        std::thread::sleep(Duration::from_millis(100));
    }

    // 本番計測
    let mut times = Vec::with_capacity(BENCH_ITERATIONS);
    for i in 0..BENCH_ITERATIONS {
        let duration = bench_fn();
        let time_sec = duration.as_secs_f64();
        times.push(time_sec);
        println!("  試行 {:2}: {:.6} s", i + 1, time_sec);

        std::thread::sleep(Duration::from_millis(50));
    }

    // 統計計算
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min = times[0];
    let max = times[BENCH_ITERATIONS - 1];
    let sum: f64 = times.iter().sum();
    let avg = sum / BENCH_ITERATIONS as f64;

    println!("  ---");
    println!("  最小値:   {:.6} s", min);
    println!("  平均値:   {:.6} s", avg);
    println!("  最大値:   {:.6} s", max);
    println!();
}

fn run_single() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    jacobi_step(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    jacobi_step(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_unsafe_semaphore() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    unsafe_semaphore(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    unsafe_semaphore(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_safe_semaphore_optimized() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    semaphore_optimized(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    semaphore_optimized(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_barrier_parallel_02() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_parallel(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    barrier_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_barrier_unsafe() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_unsafe(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    barrier_unsafe(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_rayon_v2() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    rayon_parallel(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    rayon_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_rayon_unsafe() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    rayon_unsafe(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    rayon_unsafe(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

// fn run_debug() -> Duration {
//     let mut grid_a = Grid::new();
//     let mut grid_b = Grid::new();

//     let start = Instant::now();
//     rayon_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
//     start.elapsed()
// }