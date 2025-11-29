use std::thread;
use std::sync::{Arc, Barrier, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};
/*
  両方のスレッドが同じMutexをロックしようとするため、実質的にシリアル実行になっています。

  動作の流れ

  1. スレッド1が grid_a と grid_b をロック → 上半分を計算
  2. スレッド1がロック解放
  3. スレッド2が grid_a と grid_b をロック → 下半分を計算
  4. スレッド2がロック解放
  5. バリアで同期
*/

pub fn barrier_parallel(a: &mut Grid, b: &mut Grid, steps: usize){
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    let grid_a = Arc::new(Mutex::new(a));
    let grid_b = Arc::new(Mutex::new(b));

    let barrier = Arc::new(Barrier::new(2));

    thread::scope(|scope| {
        // Thread 1: 上半分を処理
        {
            let grid_a = grid_a.clone();
            let grid_b = grid_b.clone();
            let barrier = barrier.clone();

            scope.spawn(move || {
                for step in 0..steps {
                    let (src, dst) = if step % 2 == 0 {
                        (&grid_a, &grid_b)
                    } else {
                        (&grid_b, &grid_a)
                    };
                    {
                    let src_guard = src.lock().unwrap();
                    let mut dst_guard = dst.lock().unwrap();

                    // 上半分を処理 (境界は除く)
                    for i in 1..mid {
                        for j in 1..M - 1 {
                            let idx = i * M + j;
                            let laplacian = src_guard.data[idx + M]     // i+1
                                + src_guard.data[idx - M]                // i-1
                                + src_guard.data[idx + 1]                // j+1
                                + src_guard.data[idx - 1]                // j-1
                                - 4.0 * src_guard.data[idx];

                            dst_guard.data[idx] = src_guard.data[idx] + factor * laplacian;
                        }
                    }

                    // 熱源位置を固定温度に設定（上半分に含まれる場合）
                    if N/2 < mid {
                        dst_guard.data[(N/2) * M + M/2] = 100.0;
                    }
                }
                    barrier.wait();
                }
            });
        }

        // Thread 2: 下半分を処理
        {
            let grid_a = grid_a.clone();
            let grid_b = grid_b.clone();
            let barrier = barrier.clone();

            scope.spawn(move || {
                for step in 0..steps {
                    let (src, dst) = if step % 2 == 0 {
                        (&grid_a, &grid_b)
                    } else {
                        (&grid_b, &grid_a)
                    };

                    {
                    let src_guard = src.lock().unwrap();
                    let mut dst_guard = dst.lock().unwrap();

                    // 下半分を処理 (境界は除く)
                    for i in mid..N - 1 {
                        for j in 1..M - 1 {
                            let idx = i * M + j;
                            let laplacian = src_guard.data[idx + M]     // i+1
                                + src_guard.data[idx - M]                // i-1
                                + src_guard.data[idx + 1]                // j+1
                                + src_guard.data[idx - 1]                // j-1
                                - 4.0 * src_guard.data[idx];

                            dst_guard.data[idx] = src_guard.data[idx] + factor * laplacian;
                        }
                    }

                    // 熱源位置を固定温度に設定（下半分に含まれる場合）
                    if N/2 >= mid {
                        dst_guard.data[(N/2) * M + M/2] = 100.0;
                    }
                }
                    barrier.wait();
                }
            });
        }
    });
}
