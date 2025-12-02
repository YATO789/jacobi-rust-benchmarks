use std::ptr;
use std::sync::{Arc, Barrier};
use std::thread;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

// ラッパー構造体
#[derive(Clone, Copy)]
struct GridPtr {
    data: *mut f64,
}

// ラッパーに対して Send / Sync を許可する
unsafe impl Send for GridPtr {}
unsafe impl Sync for GridPtr {}

impl GridPtr {
    // 【重要】メソッド経由でポインタを取得する
    // これにより、クロージャは "dataフィールド" ではなく "GridPtr構造体(self)" をキャプチャするようになる
    fn as_ptr(self) -> *mut f64 {
        self.data
    }
}

pub fn barrier_unsafe(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    let ptr_a = GridPtr { data: grid_a.data.as_mut_ptr() };
    let ptr_b = GridPtr { data: grid_b.data.as_mut_ptr() };

    let barrier = Arc::new(Barrier::new(2));

    thread::scope(|scope| {
        // --- スレッド1 ---
        let b1 = barrier.clone();
        scope.spawn(move || {
            // ここで .data を直接触らず、メソッド経由にする
            let mut src = ptr_a.as_ptr();
            let mut dst = ptr_b.as_ptr();

            for _step in 0..steps {
                unsafe {
                    jacobi_band_raw(src, dst, 1, mid, factor, false);
                }
                b1.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });

        // --- スレッド2 ---
        let b2 = barrier.clone();
        scope.spawn(move || {
            // こちらも同様
            let mut src = ptr_a.as_ptr();
            let mut dst = ptr_b.as_ptr();

            for _step in 0..steps {
                unsafe {
                    jacobi_band_raw(src, dst, mid, N - 1, factor, true);
                }
                b2.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });
    });

    if steps % 2 == 1 {
        unsafe {
            ptr::copy_nonoverlapping(ptr_b.as_ptr(), ptr_a.as_ptr(), N * M);
        }
    }
}

// 計算ロジック (変更なし)
#[inline(always)]
unsafe fn jacobi_band_raw(
    src: *const f64,
    dst: *mut f64,
    row_start: usize,
    row_end: usize,
    factor: f64,
    enforce_heat_source: bool,
) {
    let center_idx = (N / 2) * M + (M / 2);

    for i in row_start..row_end {
        let curr_row_offset = i * M;
        let up_row_offset = (i - 1) * M;
        let down_row_offset = (i + 1) * M;

        // SAFETY: ポインタは有効なメモリ範囲内を指している
        unsafe {
            let src_curr = src.add(curr_row_offset);
            let src_up = src.add(up_row_offset);
            let src_down = src.add(down_row_offset);
            let dst_row = dst.add(curr_row_offset);

            for j in 1..M - 1 {
                let v = *src_curr.add(j);
                let laplacian = *src_curr.add(j + 1)
                              + *src_curr.add(j - 1)
                              + *src_down.add(j)
                              + *src_up.add(j)
                              - 4.0 * v;
                *dst_row.add(j) = v + factor * laplacian;
            }
        }
    }

    if enforce_heat_source {
        if center_idx >= row_start * M && center_idx < row_end * M {
            // SAFETY: center_idxは有効な範囲内
            unsafe {
                *dst.add(center_idx) = 100.0;
            }
        }
    }
}