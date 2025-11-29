use std::time::Instant;
use std::mem;
use jacobi_rust::grid::{Grid,TIME_STEPS,WARMUP_STEPS};
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_02::barrier_parallel_02;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter;

fn main(){
    run_single();
    run_semaphore();
    run_barrier_parallel();
    run_barrier_parallel_02();
}

fn run_barrier_parallel(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_parallel(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    barrier_parallel(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("バリア: {:?}", duration);
}

fn run_barrier_parallel_02(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    barrier_parallel_02(&mut grid_a, &mut grid_b,WARMUP_STEPS);

    println!("計測開始");
    let start = Instant::now();
    barrier_parallel_02(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("バリア02 (境界共有): {:?}", duration);
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

fn run_semaphore(){
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    jacobi_steps_parallel_counter(&mut grid_a, &mut grid_b,WARMUP_STEPS);
    
    println!("計測開始");
    let start = Instant::now();
    jacobi_steps_parallel_counter(&mut grid_a, &mut grid_b, TIME_STEPS);
    let duration = start.elapsed();
    println!("unsafe semaphore: {:?}", duration);
}