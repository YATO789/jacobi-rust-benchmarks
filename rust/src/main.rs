use std::time::Instant;
use std::mem;
use jacobi_rust::grid::{Grid,TIME_STEPS,WARMUP_STEPS};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_02::barrier_parallel_02;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_03::barrier_parallel_03;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter as unsafe_semaphore;
use jacobi_rust::implementations::safe::semaphore::semaphore::jacobi_steps_parallel_counter as safe_semaphore;
use jacobi_rust::implementations::safe::semaphore::semaphore_optimized::semaphore_optimized;
use jacobi_rust::implementations::safe::rayon::{rayon_parallel, rayon_parallel_v2};

fn main(){
    println!("=== Jacobi法 2D熱方程式ベンチマーク ===\n");

    run_single();
    println!();

    run_unsafe_semaphore();
    println!();

    run_safe_semaphore();
    println!();

    run_safe_semaphore_optimized();
    println!();

    run_barrier_parallel_02();
    println!();

    run_barrier_parallel_03();
    println!();

    run_rayon();
    println!();

    run_rayon_v2();
    println!();

    println!("=== ベンチマーク完了 ===");
}



fn run_barrier_parallel_02(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_parallel_02(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    barrier_parallel_02(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("バリア02 (バリア2回): {:?}", duration);
}

fn run_barrier_parallel_03(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_parallel_03(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    barrier_parallel_03(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("バリア03 (バリア1回): {:?}", duration);
}


fn run_single(){
        {
        let mut grid_a = Grid::new();
        let mut grid_b = Grid::new();
    
        for _ in 0..WARMUP_STEPS {
            jacobi_step(&grid_a, &mut grid_b);
            mem::swap(&mut grid_a, &mut grid_b);
        }
    
        println!("計測開始");
        let start = Instant::now();
        for _ in 0..TIME_STEPS{
        jacobi_step(&grid_a, &mut grid_b);
        mem::swap(&mut grid_a, &mut grid_b);
        }
        let duration = start.elapsed();
        println!("シングル mem swap: {:?}", duration);
        }
}

fn run_unsafe_semaphore(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    unsafe_semaphore(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    unsafe_semaphore(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("unsafe semaphore: {:?}", duration);
}

fn run_safe_semaphore(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    safe_semaphore(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    safe_semaphore(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("safe semaphore (naive): {:?}", duration);
}

fn run_safe_semaphore_optimized(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    semaphore_optimized(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    semaphore_optimized(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("safe semaphore (optimized): {:?}", duration);
}

fn run_rayon(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    rayon_parallel(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    rayon_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("Rayon (基本版): {:?}", duration);
}

fn run_rayon_v2(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    rayon_parallel_v2(&mut grid_a, &mut grid_b, WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    rayon_parallel_v2(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("Rayon v2 (最適化版): {:?}", duration);
}