use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::ptr;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

// UnsafeCellをスレッド間で共有するためのラッパー
struct UnsafeSync<T>(UnsafeCell<T>);
unsafe impl<T> Sync for UnsafeSync<T> {}

pub fn unsafe_optimized(a: &mut Grid, b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    // 1. ゼロコピー分割
    let (a_upper, a_lower) = a.data.split_at_mut(mid * M);
    let (b_upper, b_lower) = b.data.split_at_mut(mid * M);

    // 2. Unsafeな共有バッファ (Mutexなし)
    // OSのロック機構を経由せず、生メモリとして扱う
    let boundary_mid_minus_1 = Arc::new(UnsafeSync(UnsafeCell::new(vec![0.0; M])));
    let boundary_mid = Arc::new(UnsafeSync(UnsafeCell::new(vec![0.0; M])));

    // 3. アトミック同期用フラグ
    let upper_ready = Arc::new(AtomicUsize::new(0));
    let lower_ready = Arc::new(AtomicUsize::new(0));
    let upper_done = Arc::new(AtomicUsize::new(0));
    let lower_done = Arc::new(AtomicUsize::new(0));

    thread::scope(|scope| {
        // --- 上半分スレッド ---
        let u_ready = upper_ready.clone();
        let l_ready = lower_ready.clone();
        let u_done = upper_done.clone();
        let l_done = lower_done.clone();
        let my_bound = boundary_mid_minus_1.clone();
        let peer_bound = boundary_mid.clone();

        scope.spawn(move || {
            let mut src = a_upper;
            let mut dst = b_upper;
            let rows = mid;
            // 固定熱源のインデックスを事前計算
            let heat_source_idx = if N / 2 < mid { Some((N / 2) * M + M / 2) } else { None };

            for step in 1..=steps {
                unsafe {
                    // 【Unsafe】生ポインタを使った高速コピー (memcpy)
                    let src_ptr = src.as_ptr();
                    let bound_ptr = (*my_bound.0.get()).as_mut_ptr();
                    // 自分の最下行をバッファへコピー
                    ptr::copy_nonoverlapping(src_ptr.add((rows - 1) * M), bound_ptr, M);
                }
                
                u_ready.store(step, Ordering::Release);
                while l_ready.load(Ordering::Acquire) < step { std::hint::spin_loop(); }

                unsafe {
                    let src_ptr = src.as_ptr();
                    let dst_ptr = dst.as_mut_ptr();
                    let peer_bound_ptr = (*peer_bound.0.get()).as_ptr();

                    // 【Unsafe】境界チェックなしの計算ループ
                    // コンパイラがSIMD命令(AVX/SSE)を生成しやすくなる
                    for i in 1..rows - 1 {
                        let curr_row_ptr = src_ptr.add(i * M);
                        let up_row_ptr = src_ptr.add((i - 1) * M);
                        let down_row_ptr = src_ptr.add((i + 1) * M);
                        let dst_row_ptr = dst_ptr.add(i * M);

                        for j in 1..M - 1 {
                            // ポインタオフセットでアクセス
                            let v = *curr_row_ptr.add(j);
                            let laplacian = *curr_row_ptr.add(j + 1)
                                          + *curr_row_ptr.add(j - 1)
                                          + *down_row_ptr.add(j)
                                          + *up_row_ptr.add(j)
                                          - 4.0 * v;
                            *dst_row_ptr.add(j) = v + factor * laplacian;
                        }
                    }

                    // 境界行（下端）の処理: 相手のバッファ(peer_bound_ptr)を読む
                    let i = rows - 1;
                    let curr_row_ptr = src_ptr.add(i * M);
                    let up_row_ptr = src_ptr.add((i - 1) * M);
                    let dst_row_ptr = dst_ptr.add(i * M);

                    for j in 1..M - 1 {
                        let v = *curr_row_ptr.add(j);
                        let down_val = *peer_bound_ptr.add(j); // 共有バッファから直接読み込み

                        let laplacian = *curr_row_ptr.add(j + 1)
                                      + *curr_row_ptr.add(j - 1)
                                      + down_val
                                      + *up_row_ptr.add(j)
                                      - 4.0 * v;
                        *dst_row_ptr.add(j) = v + factor * laplacian;
                    }

                    if let Some(idx) = heat_source_idx {
                        *dst_ptr.add(idx) = 100.0;
                    }
                }

                u_done.store(step, Ordering::Release);
                while l_done.load(Ordering::Acquire) < step { std::hint::spin_loop(); }

                std::mem::swap(&mut src, &mut dst);
            }
        });

        // --- 下半分スレッド ---
        let u_ready = upper_ready.clone();
        let l_ready = lower_ready.clone();
        let u_done = upper_done.clone();
        let l_done = lower_done.clone();
        let my_bound = boundary_mid.clone();
        let peer_bound = boundary_mid_minus_1.clone();

        scope.spawn(move || {
            let mut src = a_lower;
            let mut dst = b_lower;
            let rows = N - mid;
            let heat_source_idx = if N / 2 >= mid { Some((N / 2 - mid) * M + M / 2) } else { None };

            for step in 1..=steps {
                unsafe {
                    let src_ptr = src.as_ptr();
                    let bound_ptr = (*my_bound.0.get()).as_mut_ptr();
                    // 自分の最上行(0行目)をバッファへコピー
                    ptr::copy_nonoverlapping(src_ptr, bound_ptr, M);
                }

                l_ready.store(step, Ordering::Release);
                while u_ready.load(Ordering::Acquire) < step { std::hint::spin_loop(); }

                unsafe {
                    let src_ptr = src.as_ptr();
                    let dst_ptr = dst.as_mut_ptr();
                    let peer_bound_ptr = (*peer_bound.0.get()).as_ptr();

                    // 内部行
                    for i in 1..rows - 1 {
                        let curr_row_ptr = src_ptr.add(i * M);
                        let up_row_ptr = src_ptr.add((i - 1) * M);
                        let down_row_ptr = src_ptr.add((i + 1) * M);
                        let dst_row_ptr = dst_ptr.add(i * M);

                        for j in 1..M - 1 {
                            let v = *curr_row_ptr.add(j);
                            let laplacian = *curr_row_ptr.add(j + 1)
                                          + *curr_row_ptr.add(j - 1)
                                          + *down_row_ptr.add(j)
                                          + *up_row_ptr.add(j)
                                          - 4.0 * v;
                            *dst_row_ptr.add(j) = v + factor * laplacian;
                        }
                    }

                    // 境界行（上端）: 相手のバッファを読む
                    let _i = 0;
                    let curr_row_ptr = src_ptr; // i=0なのでオフセットなし
                    let down_row_ptr = src_ptr.add(M);
                    let dst_row_ptr = dst_ptr;

                    for j in 1..M - 1 {
                        let v = *curr_row_ptr.add(j);
                        let up_val = *peer_bound_ptr.add(j); // 共有バッファ

                        let laplacian = *curr_row_ptr.add(j + 1)
                                      + *curr_row_ptr.add(j - 1)
                                      + *down_row_ptr.add(j)
                                      + up_val
                                      - 4.0 * v;
                        *dst_row_ptr.add(j) = v + factor * laplacian;
                    }

                    if let Some(idx) = heat_source_idx {
                        *dst_ptr.add(idx) = 100.0;
                    }
                }

                l_done.store(step, Ordering::Release);
                while u_done.load(Ordering::Acquire) < step { std::hint::spin_loop(); }

                std::mem::swap(&mut src, &mut dst);
            }
        });
    });

    if steps % 2 == 1 {
        // ここも高速コピーを使用
        unsafe {
            ptr::copy_nonoverlapping(b.data.as_ptr(), a.data.as_mut_ptr(), N * M);
        }
    }
}