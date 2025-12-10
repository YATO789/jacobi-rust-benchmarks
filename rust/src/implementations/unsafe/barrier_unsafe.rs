// lib.rs (または main.rs)
use std::ptr;
use std::sync::{Arc, Barrier};
use std::thread;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

// ポインタをスレッド間で安全に渡すためのラッパー
#[derive(Clone, Copy)]
struct GridPtr {
    data: *mut f64,
}
unsafe impl Send for GridPtr {}
unsafe impl Sync for GridPtr {}

impl GridPtr {
    fn as_ptr(self) -> *mut f64 {
        self.data
    }
}

pub fn barrier_unsafe(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // Grid構造体の生ポインタを取得
    let ptr_a = GridPtr { data: grid_a.data.as_mut_ptr() };
    let ptr_b = GridPtr { data: grid_b.data.as_mut_ptr() };

    let barrier = Arc::new(Barrier::new(2));

    thread::scope(|scope| {
        // --- スレッド1: 上半分 (Rows 0 to mid) ---
        let b1 = barrier.clone();
        scope.spawn(move || {
            let mut src = ptr_a.as_ptr();
            let mut dst = ptr_b.as_ptr();

            for _step in 0..steps {
                unsafe {
                    // [1, mid) を計算。0行目は境界条件（固定）で計算しない。
                    // mid行目はスレッド2が担当するため、midは含まない。
                    // jacobi_band_raw の end_row は 排他的なので mid
                    jacobi_band_raw(src, dst, 1, mid, factor, false);
                }
                b1.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });

        // --- スレッド2: 下半分 (Rows mid to N) ---
        let b2 = barrier.clone();
        scope.spawn(move || {
            let mut src = ptr_a.as_ptr();
            let mut dst = ptr_b.as_ptr();

            for _step in 0..steps {
                unsafe {
                    // [mid, N-1) を計算。N-1行目は境界条件（固定）で計算しない。
                    // jacobi_band_raw の end_row は 排他的なので N-1
                    // N-1 は計算対象外なので、N-1で終わりたいが、jacobi_band_rawは N-1 の上まで計算するので N
                    // (N-1行目を計算するなら N, N-2行目までなら N-1)
                    jacobi_band_raw(src, dst, mid, N, factor, true);
                }
                b2.wait();
                std::mem::swap(&mut src, &mut dst);
            }
        });
    });

    if steps % 2 == 1 {
        // ステップ数が奇数の場合、b の結果を a にコピー
        unsafe {
            ptr::copy_nonoverlapping(ptr_b.as_ptr(), ptr_a.as_ptr(), N * M);
        }
    }
}

// 計算ロジック
// row_end は計算範囲の排他的な終了行
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
        // 境界行 (0, N-1) は計算しないため、i は [1, N-2] の範囲にあるはず
        if i == 0 || i == N - 1 {
            continue;
        }

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

    // 熱源の処理
    if enforce_heat_source {
        let center_row = N / 2;
        if center_row >= row_start && center_row < row_end {
            // SAFETY: center_idxは有効な範囲内
            unsafe {
                *dst.add(center_idx) = 100.0;
            }
        }
    }
}