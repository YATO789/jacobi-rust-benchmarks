use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

pub fn jacobi_steps_parallel_counter(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // グリッドをArc<Mutex<Grid>>で共有（安全に共有可変アクセスを可能にする）
    // 読み取り時もMutexを使用するが、各スレッドが異なる領域を処理するため競合は発生しない
    let read_grid_a = Arc::new(Mutex::new(grid_a.clone()));
    let write_grid_a = Arc::new(Mutex::new(grid_b.clone()));

    // カウンターで同期
    let a_counter = Arc::new(AtomicUsize::new(0));
    let b_counter = Arc::new(AtomicUsize::new(0));

    thread::scope(|scope| {
        let a_counter_clone = a_counter.clone();
        let b_counter_clone = b_counter.clone();
        let read_grid_a_clone = read_grid_a.clone();
        let write_grid_a_clone = write_grid_a.clone();

        scope.spawn(move || {
            let mut current_read = read_grid_a_clone.clone();
            let mut current_write = write_grid_a_clone.clone();

            for step in 0..steps {
                // Thread2の前回ステップ完了を待つ
                while b_counter_clone.load(Ordering::Acquire) < step {
                    std::hint::spin_loop();
                }

                // 読み取り用グリッドからデータを安全に取得
                let read_data = {
                    let read_guard = current_read.lock().unwrap();
                    read_guard.data.clone()
                };

                // 書き込み用グリッドをロックして更新
                {
                    let mut write_guard = current_write.lock().unwrap();
                    let write_data = &mut write_guard.data;

                    // 上半分を処理
                    for i in 1..mid {
                        for j in 1..M - 1 {
                            let idx = i * M + j;
                            let center = read_data[idx];
                            let laplacian = read_data[idx + M]      // i+1
                                + read_data[idx - M]                 // i-1
                                + read_data[idx + 1]                 // j+1
                                + read_data[idx - 1]                 // j-1
                                - 4.0 * center;

                            write_data[idx] = center + factor * laplacian;
                        }
                    }
                }

                // 自分の作業完了を通知
                a_counter_clone.store(step + 1, Ordering::Release);

                // 読み取り用と書き込み用をスワップ
                std::mem::swap(&mut current_read, &mut current_write);
            }
        });

        let read_grid_b_clone = read_grid_a.clone();
        let write_grid_b_clone = write_grid_a.clone();

        scope.spawn(move || {
            let mut current_read = read_grid_b_clone.clone();
            let mut current_write = write_grid_b_clone.clone();

            for step in 0..steps {
                // Thread1の前回ステップ完了を待つ
                while a_counter.load(Ordering::Acquire) < step {
                    std::hint::spin_loop();
                }

                // 読み取り用グリッドからデータを安全に取得
                let read_data = {
                    let read_guard = current_read.lock().unwrap();
                    read_guard.data.clone()
                };

                // 書き込み用グリッドをロックして更新
                {
                    let mut write_guard = current_write.lock().unwrap();
                    let write_data = &mut write_guard.data;

                    // 下半分を処理
                    for i in mid..N - 1 {
                        for j in 1..M - 1 {
                            let idx = i * M + j;
                            let center = read_data[idx];
                            let laplacian = read_data[idx + M]      // i+1
                                + read_data[idx - M]                 // i-1
                                + read_data[idx + 1]                 // j+1
                                + read_data[idx - 1]                 // j-1
                                - 4.0 * center;

                            write_data[idx] = center + factor * laplacian;
                        }
                    }

                    // 中央値を固定
                    let center_idx = (N / 2) * M + (M / 2);
                    write_data[center_idx] = 100.0;
                }

                // 自分の作業完了を通知
                b_counter.store(step + 1, Ordering::Release);

                // 読み取り用と書き込み用をスワップ
                std::mem::swap(&mut current_read, &mut current_write);
            }
        });
    });

    // 最終的な結果を元のグリッドにコピー
    // ステップ数が偶数の場合、grid_aが最終結果、奇数の場合、grid_bが最終結果
    if steps % 2 == 0 {
        *grid_a = read_grid_a.lock().unwrap().clone();
        *grid_b = write_grid_a.lock().unwrap().clone();
    } else {
        *grid_a = write_grid_a.lock().unwrap().clone();
        *grid_b = read_grid_a.lock().unwrap().clone();
    }
}
