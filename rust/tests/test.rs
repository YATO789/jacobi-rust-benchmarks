use jacobi_rust::grid::{Grid, N, M};

// main.rsで使用されているすべての実装をインポート
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::safe::rayon::rayon::rayon_parallel;
use jacobi_rust::implementations::safe::semaphore::semaphore_optimized::semaphore_optimized;
use jacobi_rust::implementations::safe::channel::channel::channel_parallel;
use jacobi_rust::implementations::unsafe_impl::parallel_unsafe::unsafe_optimized;

const TEST_STEPS: usize = 10;
const EPSILON: f64 = 1e-10;

/// グリッドの全要素が一致するかチェック
fn grids_are_equal(grid1: &Grid, grid2: &Grid) -> bool {
    if grid1.data.len() != grid2.data.len() {
        return false;
    }

    for i in 0..grid1.data.len() {
        let diff = (grid1.data[i] - grid2.data[i]).abs();
        if diff > EPSILON {
            eprintln!(
                "Mismatch at index {}: {} vs {} (diff: {})",
                i, grid1.data[i], grid2.data[i], diff
            );
            return false;
        }
    }

    true
}

/// 最終的な結果がどちらのグリッドにあるか判定して返すヘルパー
fn get_final_grid<'a>(grid_a: &'a Grid, grid_b: &'a Grid) -> &'a Grid {
    if TEST_STEPS % 2 == 0 {
        grid_a
    } else {
        grid_b
    }
}

#[test]
fn test_single_vs_unsafe_semaphore() {
    // シングルスレッド版 (正解データ)
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Unsafe セマフォ版
    let mut sem_a = Grid::new();
    let mut sem_b = Grid::new();
    jacobi_steps_parallel_counter(&mut sem_a, &mut sem_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_sem = get_final_grid(&sem_a, &sem_b);

    assert!(
        grids_are_equal(final_single, final_sem),
        "Single-thread and Unsafe Semaphore implementations produce different results"
    );

    println!("✓ Single vs Unsafe Semaphore: Results match!");
}

#[test]
fn test_single_vs_safe_semaphore_optimized() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Safe セマフォ最適化版
    let mut sem_opt_a = Grid::new();
    let mut sem_opt_b = Grid::new();
    semaphore_optimized(&mut sem_opt_a, &mut sem_opt_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_sem_opt = get_final_grid(&sem_opt_a, &sem_opt_b);

    assert!(
        grids_are_equal(final_single, final_sem_opt),
        "Single-thread and Safe Semaphore Optimized implementations produce different results"
    );

    println!("✓ Single vs Safe Semaphore Optimized: Results match!");
}

#[test]
fn test_single_vs_barrier_parallel() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // バリア並列版
    let mut barrier_a = Grid::new();
    let mut barrier_b = Grid::new();
    barrier_parallel(&mut barrier_a, &mut barrier_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_barrier = get_final_grid(&barrier_a, &barrier_b);

    assert!(
        grids_are_equal(final_single, final_barrier),
        "Single-thread and Barrier Parallel implementations produce different results"
    );

    println!("✓ Single vs Barrier Parallel: Results match!");
}

#[test]
fn test_single_vs_rayon_v2() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Rayon v2版
    let mut rayon_a = Grid::new();
    let mut rayon_b = Grid::new();
    rayon_parallel(&mut rayon_a, &mut rayon_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_rayon = get_final_grid(&rayon_a, &rayon_b);

    assert!(
        grids_are_equal(final_single, final_rayon),
        "Single-thread and Rayon v2 implementations produce different results"
    );

    println!("✓ Single vs Rayon v2: Results match!");
}

#[test]
fn test_single_vs_channel() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Channel版
    let mut channel_a = Grid::new();
    let mut channel_b = Grid::new();
    channel_parallel(&mut channel_a, &mut channel_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_channel = get_final_grid(&channel_a, &channel_b);

    assert!(
        grids_are_equal(final_single, final_channel),
        "Single-thread and Channel implementations produce different results"
    );

    println!("✓ Single vs Channel: Results match!");
}

#[test]
fn test_single_vs_unsafe_optimized() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Unsafe Optimized版
    let mut unsafe_opt_a = Grid::new();
    let mut unsafe_opt_b = Grid::new();
    unsafe_optimized(&mut unsafe_opt_a, &mut unsafe_opt_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_unsafe = get_final_grid(&unsafe_opt_a, &unsafe_opt_b);

    assert!(
        grids_are_equal(final_single, final_unsafe),
        "Single-thread and Unsafe Optimized implementations produce different results"
    );

    println!("✓ Single vs Unsafe Optimized: Results match!");
}

#[test]
fn test_single_step_consistency() {
    // 同じ初期条件で2回実行して結果が同じか確認（決定論的であることの確認）
    let mut grid1_a = Grid::new();
    let mut grid1_b = Grid::new();
    jacobi_step(&mut grid1_a, &mut grid1_b, TEST_STEPS);

    let mut grid2_a = Grid::new();
    let mut grid2_b = Grid::new();
    jacobi_step(&mut grid2_a, &mut grid2_b, TEST_STEPS);

    let final1 = get_final_grid(&grid1_a, &grid1_b);
    let final2 = get_final_grid(&grid2_a, &grid2_b);

    assert!(
        grids_are_equal(final1, final2),
        "Single-thread implementation is not deterministic"
    );

    println!("✓ Single-thread consistency: Results match!");
}

#[test]
fn test_heat_source_preserved() {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    // 複数ステップ実行
    jacobi_step(&mut grid_a, &mut grid_b, TEST_STEPS);

    let final_grid = get_final_grid(&grid_a, &grid_b);

    // 熱源位置(N/2, M/2)が100.0のまま保持されているか確認
    let heat_source_idx = N / 2 * M + M / 2;
    assert_eq!(
        final_grid.data[heat_source_idx], 100.0,
        "Heat source at ({}, {}) should remain 100.0, but got {}",
        N / 2, M / 2, final_grid.data[heat_source_idx]
    );

    println!("✓ Heat source preserved: 100.0 at center!");
}

#[test]
fn test_boundary_conditions() {
    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    // 複数ステップ実行
    jacobi_step(&mut grid_a, &mut grid_b, TEST_STEPS);

    let final_grid = get_final_grid(&grid_a, &grid_b);

    // 境界が0.0のまま保持されているか確認
    // 上境界 (j=0..M) ※i=0
    for j in 0..M {
        let idx = j;
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Top boundary at (0, {}) should be 0.0, but got {}",
            j, final_grid.data[idx]
        );
    }

    // 下境界 (j=0..M) ※i=N-1
    for j in 0..M {
        let idx = (N - 1) * M + j;
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Bottom boundary at ({}, {}) should be 0.0, but got {}",
            N - 1, j, final_grid.data[idx]
        );
    }

    // 左境界 (i=0..N) ※j=0
    for i in 0..N {
        let idx = i * M;
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Left boundary at ({}, 0) should be 0.0, but got {}",
            i, final_grid.data[idx]
        );
    }

    // 右境界 (i=0..N) ※j=M-1
    for i in 0..N {
        let idx = i * M + (M - 1);
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Right boundary at ({}, {}) should be 0.0, but got {}",
            i, M - 1, final_grid.data[idx]
        );
    }

    println!("✓ Boundary conditions: All boundaries remain 0.0!");
}