use std::time::Instant;
use jacobi_rust::grid::{Grid, TIME_STEPS};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter as unsafe_semaphore;
use jacobi_rust::implementations::safe::semaphore::semaphore_optimized::semaphore_optimized;
use jacobi_rust::implementations::safe::rayon::rayon::rayon_parallel;
use jacobi_rust::implementations::unsafe_impl::barrier_unsafe::barrier_unsafe;
use jacobi_rust::implementations::unsafe_impl::rayon_unsafe::rayon_unsafe;

const BENCH_ITERATIONS: usize = 10;
const BENCH_WARMUP: usize = 5;

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

    bench("Single Thread", |a, b| jacobi_step(a, b, TIME_STEPS));
    bench("Unsafe Semaphore", |a, b| unsafe_semaphore(a, b, TIME_STEPS));
    bench("Safe Semaphore", |a, b| semaphore_optimized(a, b, TIME_STEPS));
    bench("Barrier", |a, b| barrier_parallel(a, b, TIME_STEPS));
    bench("Barrier Unsafe", |a, b| barrier_unsafe(a, b, TIME_STEPS));
    bench("Rayon", |a, b| rayon_parallel(a, b, TIME_STEPS));
    bench("Rayon Unsafe", |a, b| rayon_unsafe(a, b, TIME_STEPS));

    println!("\n=== ベンチマーク完了 ===");
}

fn bench<F: Fn(&mut Grid, &mut Grid)>(label: &str, func: F) {
    let mut times = Vec::new();

    for _ in 0..BENCH_WARMUP {
        let mut a = Grid::new();
        let mut b = Grid::new();
        func(&mut a, &mut b);
    }

    for _ in 0..BENCH_ITERATIONS {
        let mut a = Grid::new();
        let mut b = Grid::new();

        let start = Instant::now();
        func(&mut a, &mut b);
        let t = start.elapsed().as_secs_f64();
        times.push(t);
    }

    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{label}: min={:.6}, avg={:.6}, max={:.6}", 
        times[0], 
        times.iter().sum::<f64>() / times.len() as f64, 
        times[times.len() - 1]
    );
}