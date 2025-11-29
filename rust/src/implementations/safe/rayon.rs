use rayon::prelude::*;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/*
  Rayon-based parallel implementation

  Key features:
  1. Uses Rayon's work-stealing thread pool for automatic load balancing
  2. Safe parallel iteration with par_iter and collect
  3. Each row is computed independently in parallel
  4. No manual thread management or synchronization needed
*/

pub fn rayon_parallel(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    for step in 0..steps {
        let (src, dst) = if step % 2 == 0 {
            (&a.data[..], &mut b.data[..])
        } else {
            (&b.data[..], &mut a.data[..])
        };

        // Copy boundary rows
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        // Compute interior points in parallel using row-level parallelism
        let results: Vec<Vec<f64>> = (1..N - 1)
            .into_par_iter()
            .map(|i| {
                let mut row_result = vec![0.0; M];

                // Boundaries stay the same
                row_result[0] = src[i * M];
                row_result[M - 1] = src[i * M + M - 1];

                // Compute interior points
                for j in 1..M - 1 {
                    let idx = i * M + j;
                    let laplacian = src[idx + M]
                        + src[idx - M]
                        + src[idx + 1]
                        + src[idx - 1]
                        - 4.0 * src[idx];

                    row_result[j] = src[idx] + factor * laplacian;
                }

                row_result
            })
            .collect();

        // Write computed results back
        for (row_idx, row) in results.iter().enumerate() {
            let i = row_idx + 1;
            dst[i * M..(i + 1) * M].copy_from_slice(row);
        }

        // Set heat source to fixed temperature
        dst[(N / 2) * M + M / 2] = 100.0;
    }

    // Ensure final result is in grid a
    if steps % 2 == 1 {
        a.data.copy_from_slice(&b.data);
    }
}

/*
  Optimized version using par_chunks_mut for row-level parallelism
  This is the most efficient safe implementation
*/
pub fn rayon_parallel_v2(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);

    for step in 0..steps {
        let (src, dst) = if step % 2 == 0 {
            (&a.data[..], &mut b.data[..])
        } else {
            (&b.data[..], &mut a.data[..])
        };

        // Copy boundaries first
        dst[0..M].copy_from_slice(&src[0..M]);
        dst[(N - 1) * M..N * M].copy_from_slice(&src[(N - 1) * M..N * M]);

        // Process interior rows in parallel
        // We need to split the destination slice to allow parallel mutable access
        let interior_start = M;
        let interior_end = (N - 1) * M;

        dst[interior_start..interior_end]
            .par_chunks_mut(M)
            .enumerate()
            .for_each(|(row_idx, dst_row)| {
                let i = row_idx + 1; // Actual row index in the grid

                for j in 1..M - 1 {
                    let idx = i * M + j;
                    let laplacian = src[idx + M]
                        + src[idx - M]
                        + src[idx + 1]
                        + src[idx - 1]
                        - 4.0 * src[idx];

                    dst_row[j] = src[idx] + factor * laplacian;
                }

                // Copy boundary values for this row
                dst_row[0] = src[i * M];
                dst_row[M - 1] = src[i * M + M - 1];
            });

        // Set heat source to fixed temperature
        dst[(N / 2) * M + M / 2] = 100.0;
    }

    // Ensure final result is in grid a
    if steps % 2 == 1 {
        a.data.copy_from_slice(&b.data);
    }
}
