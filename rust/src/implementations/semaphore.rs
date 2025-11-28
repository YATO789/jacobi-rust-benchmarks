use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

#[repr(align(64))]
struct AlignedAtomic(AtomicUsize);

pub fn jacobi_steps_parallel_counter(grid_a: &mut Grid, grid_b: &mut Grid, steps: usize) {
    let mid = N / 2;
    let factor = ALPHA * DT / (DX * DX);

    let ptr_a = grid_a as *mut Grid as usize;
    let ptr_b = grid_b as *mut Grid as usize;

    let s11 = Arc::new(AlignedAtomic(AtomicUsize::new(0)));
    let s21 = Arc::new(AlignedAtomic(AtomicUsize::new(0)));

    thread::scope(|scope| {
        let s11_clone = s11.clone();
        let s21_clone = s21.clone();

        // Thread 1: 上半分 (1..mid)
        scope.spawn(move || {
            for step in 0..steps {
                let src = if step % 2 == 0 { ptr_a } else { ptr_b } as *const Grid;
                let dst = if step % 2 == 0 { ptr_b } else { ptr_a } as *mut Grid;

                // Thread2の前ステップ完了を待つ
                loop {
                    if s21_clone.0.load(Ordering::Relaxed) >= step {
                        std::sync::atomic::fence(Ordering::Acquire);
                        break;
                    }
                    std::hint::spin_loop();
                }

                unsafe {
                    let src_ref = &*src;
                    let dst_ref = &mut *dst;

                    for i in 1..mid {
                        for j in 1..M - 1 {
                            let idx = i * M + j;
                            let center = *src_ref.data.get_unchecked(idx);
                            let laplacian = *src_ref.data.get_unchecked((i + 1) * M + j)
                                + *src_ref.data.get_unchecked((i - 1) * M + j)
                                + *src_ref.data.get_unchecked(i * M + (j + 1))
                                + *src_ref.data.get_unchecked(i * M + (j - 1))
                                - 4.0 * center;

                            *dst_ref.data.get_unchecked_mut(idx) = center + factor * laplacian;
                        }
                    }
                }

                s11_clone.0.store(step + 1, Ordering::Release);
            }
        });

        // Thread 2: 下半分 (mid..N-1)
        scope.spawn(move || {
            for step in 0..steps {
                let src = if step % 2 == 0 { ptr_a } else { ptr_b } as *const Grid;
                let dst = if step % 2 == 0 { ptr_b } else { ptr_a } as *mut Grid;

                // Thread1の前ステップ完了を待つ
                loop {
                    if s11.0.load(Ordering::Relaxed) >= step {
                        std::sync::atomic::fence(Ordering::Acquire);
                        break;
                    }
                    std::hint::spin_loop();
                }

                unsafe {
                    let src_ref = &*src;
                    let dst_ref = &mut *dst;

                    for i in mid..N - 1 {
                        for j in 1..M - 1 {
                            // 熱源位置は固定温度
                            if i == mid && j == mid {
                                *dst_ref.data.get_unchecked_mut(i * M + j) = 100.0;
                                continue;
                            }

                            let idx = i * M + j;
                            let center = *src_ref.data.get_unchecked(idx);
                            let laplacian = *src_ref.data.get_unchecked((i + 1) * M + j)
                                + *src_ref.data.get_unchecked((i - 1) * M + j)
                                + *src_ref.data.get_unchecked(i * M + (j + 1))
                                + *src_ref.data.get_unchecked(i * M + (j - 1))
                                - 4.0 * center;

                            *dst_ref.data.get_unchecked_mut(idx) = center + factor * laplacian;
                        }
                    }
                }

                s21.0.store(step + 1, Ordering::Release);
            }
        });
    });
}