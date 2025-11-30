use jacobi_rust::grid::Grid;
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_02::barrier_parallel_02;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_03::barrier_parallel_03;
use std::mem;

fn main() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..10 {
        jacobi_step(&single_a, &mut single_b);
        mem::swap(&mut single_a, &mut single_b);
    }

    // barrier_parallel_02版
    let mut barrier02_a = Grid::new();
    let mut barrier02_b = Grid::new();
    barrier_parallel_02(&mut barrier02_a, &mut barrier02_b, 10);

    // barrier_parallel_03版
    let mut barrier03_a = Grid::new();
    let mut barrier03_b = Grid::new();
    barrier_parallel_03(&mut barrier03_a, &mut barrier03_b, 10);

    println!("Index 11 values:");
    println!("Single:      {}", single_a.data[11]);
    println!("Barrier 02:  {}", barrier02_a.data[11]);
    println!("Barrier 03:  {}", barrier03_a.data[11]);

    println!("\nFull grid comparison (first 20 elements):");
    for i in 0..20 {
        println!("Index {}: single={}, b02={}, b03={}",
            i, single_a.data[i], barrier02_a.data[i], barrier03_a.data[i]);
    }
}
