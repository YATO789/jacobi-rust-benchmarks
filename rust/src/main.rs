use std::time::Instant;
use std::mem;
use jacobi_rust::grid::{Grid,TIME_STEPS,WARMUP_STEPS};
use jacobi_rust::implementations::single::jacobi_step;

fn main(){
    run_single();
}


fn run_single(){
    {
        let mut grid_a = Grid::new();
        let mut grid_b = Grid::new();
    
        for _ in 0..WARMUP_STEPS {
            jacobi_step(&grid_a, &mut grid_b);
            jacobi_step(&grid_b, &mut grid_a);
        }
    
        println!("計測開始");
        let start = Instant::now();
        for _ in 0..TIME_STEPS {
        jacobi_step(&grid_a, &mut grid_b);
        jacobi_step(&grid_b, &mut grid_a);
        }
        let duration = start.elapsed();
        println!("実行時間: {:?}", duration);
        }
    
        {
        let mut grid_a = Grid::new();
        let mut grid_b = Grid::new();
    
        for _ in 0..WARMUP_STEPS {
            jacobi_step(&grid_a, &mut grid_b);
            mem::swap(&mut grid_a, &mut grid_b);
        }
    
        println!("計測開始");
        let start = Instant::now();
        for _ in 0..TIME_STEPS * 2 {
        jacobi_step(&grid_a, &mut grid_b);
        mem::swap(&mut grid_a, &mut grid_b);
        }
        let duration = start.elapsed();
        println!("実行時間: {:?}", duration);
        }
}