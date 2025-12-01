use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

pub fn semaphore_optimized(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // 同期用のアトミックカウンター
    // "Ready": 境界データの書き込みが完了したことを示す
    // "Done":  計算が完了し、境界バッファを解放して良いことを示す
    let upper_ready = Arc::new(AtomicUsize::new(0));
    let lower_ready = Arc::new(AtomicUsize::new(0));
    let upper_done = Arc::new(AtomicUsize::new(0));
    let lower_done = Arc::new(AtomicUsize::new(0));

    // 境界データ共有用 (MutexはUnsafe回避のためのコンテナとして使用)
    let boundary_mid_minus_1 = Arc::new(Mutex::new(vec![0.0; M])); 
    let boundary_mid = Arc::new(Mutex::new(vec![0.0; M]));

    // ゼロコピー: データを可変スライスとして分割
    let (a_upper, a_lower) = a.data.split_at_mut(mid * M);
    let (b_upper, b_lower) = b.data.split_at_mut(mid * M);

    thread::scope(|scope| {
        // --- 上半分スレッド ---
        let u_ready = upper_ready.clone();
        let l_ready = lower_ready.clone();
        let u_done = upper_done.clone();
        let l_done = lower_done.clone();
        
        let my_bound_writer = boundary_mid_minus_1.clone();
        let other_bound_reader = boundary_mid.clone();

        scope.spawn(move || {
            let mut src = a_upper;
            let mut dst = b_upper;
            let rows = mid;

            for step in 1..=steps { // stepカウントを1から開始にしてわかりやすくする Inclusive Range (以下)	1 から steps まで
                // 1. 境界データを共有バッファに書き込み
                {
                    let mut writer = my_bound_writer.lock().unwrap();
                    let row_idx = rows - 1;
                    writer.copy_from_slice(&src[row_idx * M..(row_idx + 1) * M]);
                }

                // 通知: 「データ準備よし」
                u_ready.store(step, Ordering::Release);

                // 待機: 相手のデータ準備ができるまでスピン待機
                while l_ready.load(Ordering::Acquire) < step {
                    std::hint::spin_loop();
                }

                // 2. 計算フェーズ
                // 内部
                for i in 1..rows - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx - M] + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }
                
                // 境界 (相手のバッファを読む)
                {
                    let reader = other_bound_reader.lock().unwrap();
                    let i = rows - 1;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let down_val = reader[j];
                        let laplacian = src[idx - M] + down_val + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                if N / 2 < mid {
                    dst[(N / 2) * M + M / 2] = 100.0;
                }

                // 通知: 「計算完了（バッファ読み終わった）」
                u_done.store(step, Ordering::Release);

                // 待機: 相手も計算を終えるまで待つ
                // これがないと、次のループで自分が書き込む際、相手がまだ読んでる最中かもしれない
                while l_done.load(Ordering::Acquire) < step {
                    std::hint::spin_loop();
                }

                std::mem::swap(&mut src, &mut dst);
            }
        });

        // --- 下半分スレッド ---
        let u_ready = upper_ready.clone();
        let l_ready = lower_ready.clone();
        let u_done = upper_done.clone();
        let l_done = lower_done.clone();

        let my_bound_writer = boundary_mid.clone();
        let other_bound_reader = boundary_mid_minus_1.clone();

        scope.spawn(move || {
            let mut src = a_lower;
            let mut dst = b_lower;
            let rows = N - mid;

            for step in 1..=steps {
                // 1. 境界書き込み
                {
                    let mut writer = my_bound_writer.lock().unwrap();
                    writer.copy_from_slice(&src[0..M]);
                }

                // 通知: Ready
                l_ready.store(step, Ordering::Release);

                // 待機: 相手のReady
                while u_ready.load(Ordering::Acquire) < step {
                    std::hint::spin_loop();
                }

                // 2. 計算
                for i in 1..rows - 1 {
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let laplacian = src[idx - M] + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                // 境界 (相手のバッファを読む)
                {
                    let reader = other_bound_reader.lock().unwrap();
                    let i = 0;
                    for j in 1..M - 1 {
                        let idx = i * M + j;
                        let up_val = reader[j];
                        let laplacian = up_val + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
                        dst[idx] = src[idx] + factor * laplacian;
                    }
                }

                if N / 2 >= mid {
                    let local_row = N / 2 - mid;
                    dst[local_row * M + M / 2] = 100.0;
                }

                // 通知: Done
                l_done.store(step, Ordering::Release);

                // 待機: 相手のDone
                while u_done.load(Ordering::Acquire) < step {
                    std::hint::spin_loop();
                }

                std::mem::swap(&mut src, &mut dst);
            }
        });
    });

    // 奇数ステップ終了時の書き戻し処理が必要であればここで行う
    if steps % 2 == 1 {
        a.data.copy_from_slice(&b.data);
    }
}