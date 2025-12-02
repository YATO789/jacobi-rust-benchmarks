use std::time::{Duration, Instant};
use jacobi_rust::grid::{Grid, TIME_STEPS, WARMUP_STEPS};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter as unsafe_semaphore;
use jacobi_rust::implementations::safe::semaphore::semaphore_optimized::semaphore_optimized;
use jacobi_rust::implementations::safe::rayon::rayon::rayon_parallel;
use jacobi_rust::implementations::safe::channel::channel::channel_parallel;
use jacobi_rust::implementations::unsafe_impl::parallel_unsafe::unsafe_optimized;

const BENCH_ITERATIONS: usize = 15;
const BENCH_WARMUP: usize = 3;

fn main() {
    // Rayonのスレッド数を2に制限
    rayon::ThreadPoolBuilder::new()
    .num_threads(2)
    .build_global()
    .unwrap();

    println!("Rayon スレッド数: {}", rayon::current_num_threads());

    println!("=== Jacobi法 2D熱方程式ベンチマーク ===");
    println!("TIME_STEPS: {}, 測定回数: {}\n", TIME_STEPS, BENCH_ITERATIONS);

    run_benchmark("Single Thread", run_single);
    run_benchmark("Unsafe Semaphore", run_unsafe_semaphore);
    run_benchmark("Safe Semaphore", run_safe_semaphore_optimized);
    run_benchmark("Barrier", run_barrier_parallel_02);
    // run_benchmark("Barrier Unsafe", run_barrier_unsafe);
    run_benchmark("Rayon", run_rayon_v2);
    run_benchmark("unsafe parallel", run_unsafe_opt);


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
        // キャッシュクリア（疑似的）
        let _dummy: Vec<u8> = vec![0; 5 * 1024 * 1024];

        let duration = bench_fn();
        times.push(duration);
        println!("  試行 {:2}: {:?}", i + 1, duration);

        std::thread::sleep(Duration::from_millis(50));
    }

    // 統計計算
    times.sort();
    let median = times[BENCH_ITERATIONS / 2];
    let avg = times.iter().sum::<Duration>() / BENCH_ITERATIONS as u32;
    let min = times[0];
    let max = times[BENCH_ITERATIONS - 1];

    println!("  ---");
    println!("  最小値:   {:?}", min);
    println!("  中央値:   {:?}", median);
    println!("  平均値:   {:?}", avg);
    println!("  最大値:   {:?}", max);
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

// fn run_barrier_unsafe() -> Duration {
//     let mut grid_a = Grid::new();
//     let mut grid_b = Grid::new();

//     barrier_unsafe(&mut grid_a, &mut grid_b, WARMUP_STEPS);

//     let start = Instant::now();
//     barrier_unsafe(&mut grid_a, &mut grid_b, TIME_STEPS);
//     start.elapsed()
// }

fn run_rayon_v2() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    rayon_parallel(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    rayon_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_channel_parallel() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    channel_parallel(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    channel_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}

fn run_unsafe_opt() -> Duration {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();
    // Warmup
    unsafe_optimized(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    let start = Instant::now();
    unsafe_optimized(&mut grid_a, &mut grid_b, TIME_STEPS);
    start.elapsed()
}