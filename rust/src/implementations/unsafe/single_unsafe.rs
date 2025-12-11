use std::mem;
use std::ptr;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

/// 生ポインタを使ったシングルスレッドJacobi法実装
///
/// 境界チェックを除去することでパフォーマンスを最適化した実装。
/// Safe版のsingle.rsと同じアルゴリズムだが、unsafeブロックを使用して
/// 配列アクセスの境界チェックを省略している。
pub fn jacobi_step_unsafe(a: &mut Grid, b: &mut Grid, steps: usize) {
    let factor = ALPHA * DT / (DX * DX);
    let center_idx = (N / 2) * M + (M / 2);

    let mut src = a.data.as_mut_ptr();
    let mut dst = b.data.as_mut_ptr();

    for _ in 0..steps {
        unsafe {
            // 全グリッドを計算（境界は除く）
            for i in 1..N-1 {
                let curr_row = src.add(i * M);
                let up_row = src.add((i - 1) * M);
                let down_row = src.add((i + 1) * M);
                let dst_row = dst.add(i * M);

                for j in 1..M-1 {
                    let v = *curr_row.add(j);
                    let laplacian = *curr_row.add(j + 1)
                                  + *curr_row.add(j - 1)
                                  + *down_row.add(j)
                                  + *up_row.add(j)
                                  - 4.0 * v;
                    *dst_row.add(j) = v + factor * laplacian;
                }
            }

            // 熱源位置を固定温度に設定
            *dst.add(center_idx) = 100.0;
        }

        mem::swap(&mut src, &mut dst);
    }

    // ステップ数が奇数の場合、結果をaに戻す
    if steps % 2 == 1 {
        unsafe {
            ptr::copy_nonoverlapping(src, a.data.as_mut_ptr(), N * M);
        }
    }
}
