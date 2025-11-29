use std::thread;
use std::sync::{Arc, Barrier, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Optimized parallel implementation without unsafe

  Key optimizations:
  1. Split data into upper and lower regions for independent processing
  2. Share only boundary rows via Mutex (minimize lock scope)
  3. Most computation is lock-free
  4. Use barrier synchronization to ensure step completion
  5. Minimize memory copying
*/

pub fn barrier_parallel_02(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // Shared boundary row buffers
    let upper_boundary = Arc::new(Mutex::new(vec![0.0; M])); // row mid-1
    let lower_boundary = Arc::new(Mutex::new(vec![0.0; M])); // row mid

    let barrier = Arc::new(Barrier::new(2));

    // Split data
    let upper_a: Vec<f64> = a.data[0..mid * M].to_vec();
    let upper_b: Vec<f64> = b.data[0..mid * M].to_vec();
    let lower_a: Vec<f64> = a.data[mid * M..N * M].to_vec();
    let lower_b: Vec<f64> = b.data[mid * M..N * M].to_vec();

    thread::scope(|scope| {
        // Thread 1: Process upper half (0..mid)
        let barrier1 = barrier.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let upper_handle = scope.spawn(move || {
            let mut src = upper_a;
            let mut dst = upper_b;

            for _step in 0..steps {
                // Write our boundary row (mid-1) to shared buffer BEFORE computation
                {
                    let mut upper_bound_row = upper_bound.lock().unwrap();
                    for j in 0..M {
                        upper_bound_row[j] = src[(mid - 1) * M + j];
                    }
                }

                // Barrier: ensure both threads have written their boundary rows
                barrier1.wait();

                // Compute internal rows (1..mid-2, lock-free)
                for i in 1..mid.saturating_sub(1) {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M]
                            + src[idx - M]
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // Compute boundary row (mid-1) which references lower half's row mid
                if mid >= 1 {
                    let lower_bound_row = lower_bound.lock().unwrap();
                    let i = mid - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = lower_bound_row[j]  // lower half's row 0 (row mid)
                            + src[idx - M]
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // Set heat source temperature (if in upper half)
                if N / 2 < mid {
                    dst[(N / 2) * M + M / 2] = 100.0;
                }

                // Barrier: wait for all computations to complete before swapping
                barrier1.wait();

                std::mem::swap(&mut src, &mut dst);
            }

            if steps % 2 == 0 { src } else { dst }
        });

        // Thread 2: Process lower half (mid..N)
        let barrier2 = barrier.clone();
        let upper_bound = upper_boundary.clone();
        let lower_bound = lower_boundary.clone();

        let lower_handle = scope.spawn(move || {
            let mut src = lower_a;
            let mut dst = lower_b;
            let lower_n = N - mid;

            for _step in 0..steps {
                // Write our first row (0 = row mid) to shared buffer BEFORE computation
                {
                    let mut lower_bound_row = lower_bound.lock().unwrap();
                    for j in 0..M {
                        lower_bound_row[j] = src[j];
                    }
                }

                // Barrier: ensure both threads have written their boundary rows
                barrier2.wait();

                // Compute internal rows (1..lower_n-1, lock-free)
                for i in 1..lower_n - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M]
                            + src[idx - M]
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // Compute boundary row (0 = row mid) which references upper half's row mid-1
                {
                    let upper_bound_row = upper_bound.lock().unwrap();
                    let i = 0;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx + M]
                            + upper_bound_row[j]  // upper half's row mid-1
                            + src[idx + 1]
                            + src[idx - 1]
                            - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // Set heat source temperature (if in lower half)
                if N / 2 >= mid {
                    let heat_i = N / 2 - mid;
                    dst[heat_i * M + M / 2] = 100.0;
                }

                // Barrier: wait for all computations to complete before swapping
                barrier2.wait();

                std::mem::swap(&mut src, &mut dst);
            }

            if steps % 2 == 0 { src } else { dst }
        });

        // Merge results
        let final_upper = upper_handle.join().unwrap();
        let final_lower = lower_handle.join().unwrap();

        // Write back to original grid
        a.data[0..mid * M].copy_from_slice(&final_upper);
        a.data[mid * M..N * M].copy_from_slice(&final_lower);
    });
}
