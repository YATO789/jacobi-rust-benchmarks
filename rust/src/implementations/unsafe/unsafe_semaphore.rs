// lib.rs (または main.rs)
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

// ポインタをスレッド間で安全に渡すためのラッパー (Send/Sync実装)
#[derive(Clone, Copy, Debug)]
struct GridHandle(NonNull<Grid>);
unsafe impl Send for GridHandle {}
unsafe impl Sync for GridHandle {}

#[repr(align(64))]
struct AlignedAtomic(AtomicUsize);

pub fn jacobi_steps_parallel_counter(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    let ptr_a = GridHandle(NonNull::from(grid_a));
    let ptr_b = GridHandle(NonNull::from(grid_b));

    // s_upper, s_lower は、バリアとしてステップ完了を通知するために使用
    let s_upper = Arc::new(AlignedAtomic(AtomicUsize::new(0)));
    let s_lower = Arc::new(AlignedAtomic(AtomicUsize::new(0)));

    thread::scope(|scope| {
        // Thread 1: 上半分 (1..mid)
        let l_ready = s_lower.clone();
        let u_signal = s_upper.clone();

        scope.spawn(move || {
            for step in 0..steps {
                let (src, dst) = select_buffers(step, ptr_a, ptr_b);
                
                // 1. 計算 (dstへの書き込み)
                unsafe {
                    // [1, mid) を計算。N行目は境界条件と仮定して計算しない
                    jacobi_band(src, dst, 1, mid, factor, false);
                }

                // 2. 信号: 計算完了を通知
                u_signal.0.store(step + 1, Ordering::Release);

                // 3. 待機: 相手の計算完了を待つ (バリアエミュレーション)
                wait_for_step(&l_ready, step + 1);
            }
        });

        // Thread 2: 下半分 (mid..N-1)
        let u_ready = s_upper.clone();
        let l_signal = s_lower.clone();

        scope.spawn(move || {
            for step in 0..steps {
                let (src, dst) = select_buffers(step, ptr_a, ptr_b);
                
                // 1. 計算 (dstへの書き込み)
                unsafe {
                    // [mid, N) を計算。
                    jacobi_band(src, dst, mid, N - 1, factor, true);
                }

                // 2. 信号: 計算完了を通知
                l_signal.0.store(step + 1, Ordering::Release);

                // 3. 待機: 相手の計算完了を待つ (バリアエミュレーション)
                wait_for_step(&u_ready, step + 1);
            }
        });
    });
    
    // 奇数ステップ終了時のコピー
    if steps % 2 == 1 {
        // ptr_aとptr_bはGridHandleであり、生ポインタのラッパーであるため、
        // thread::scopeのライフタイムの外で安全に使用できます。
        // これらは元のgrid_a/grid_bへの参照を保持していません。

        // GridHandleから生のGridポインタを取り出す
        let grid_a_ptr = ptr_a.0.as_ptr();
        let grid_b_ptr = ptr_b.0.as_ptr();

        unsafe {
            // Gridポインタからデータ配列(*mut f64)へのポインタを取得する
            let ptr_a_data = (*grid_a_ptr).data.as_mut_ptr();
            let ptr_b_data = (*grid_b_ptr).data.as_ptr();
            
            // ptr::copy_nonoverlapping(src, dst, count)
            ptr::copy_nonoverlapping(ptr_b_data, ptr_a_data, N * M);
        }
    }
}

// === ユーティリティ関数 ===

#[inline(always)]
fn select_buffers(step: usize, ptr_a: GridHandle, ptr_b: GridHandle) -> (GridHandle, GridHandle) {
    if step & 1 == 0 {
        (ptr_a, ptr_b) // 偶数ステップ: read A, write B
    } else {
        (ptr_b, ptr_a) // 奇数ステップ: read B, write A
    }
}

#[inline(always)]
unsafe fn jacobi_band(
    src: GridHandle,
    dst: GridHandle,
    row_start: usize,
    row_end: usize,
    factor: f64,
    enforce_heat_source: bool,
) {
    let src_ref = unsafe { src.0.as_ref() };
    let mut dst_ptr = dst.0;
    let dst_ref = unsafe { dst_ptr.as_mut() };
    let center_row = N / 2;
    let center_col = M / 2;
    let center_idx = center_row * M + center_col;

    for i in row_start..row_end {
        if enforce_heat_source && i == center_row {
            unsafe { update_row(src_ref, dst_ref, i, 1, center_col, factor) };
            unsafe { update_row(src_ref, dst_ref, i, center_col + 1, M - 1, factor) };
            continue;
        }
        unsafe { update_row(src_ref, dst_ref, i, 1, M - 1, factor) };
    }

    if enforce_heat_source {
        unsafe {
            *dst_ref.data.get_unchecked_mut(center_idx) = 100.0;
        }
    }
}

#[inline(always)]
unsafe fn update_row(
    src: &Grid,
    dst: &mut Grid,
    row: usize,
    col_start: usize,
    col_end: usize,
    factor: f64,
) {
    let src_ptr = src.data.as_ptr();
    let dst_ptr = dst.data.as_mut_ptr();

    for j in col_start..col_end {
        let idx = row * M + j;
        unsafe {
            let center = *src_ptr.add(idx);
            let laplacian = *src_ptr.add(idx + M)
                + *src_ptr.add(idx - M)
                + *src_ptr.add(idx + 1)
                + *src_ptr.add(idx - 1)
                - 4.0 * center;

            *dst_ptr.add(idx) = center + factor * laplacian;
        }
    }
}

#[inline(always)]
fn wait_for_step(counter: &AlignedAtomic, step: usize) {
    const SPIN_BEFORE_YIELD: usize = 256;
    let mut spin = 0;
    loop {
        if counter.0.load(Ordering::Relaxed) >= step {
            std::sync::atomic::fence(Ordering::Acquire);
            break;
        }
        std::hint::spin_loop();
        spin += 1;
        if spin >= SPIN_BEFORE_YIELD {
            spin = 0;
            std::thread::yield_now();
        }
    }
}