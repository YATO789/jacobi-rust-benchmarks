use jacobi_rust::grid::Grid;
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::unsafe_impl::unsafe_semaphore::jacobi_steps_parallel_counter;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_02::barrier_parallel_02;
use jacobi_rust::implementations::safe::barrier::barrier_parallel_03::barrier_parallel_03;
use jacobi_rust::implementations::safe::rayon::{rayon_parallel, rayon_parallel_v2};

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

#[test]
fn test_single_vs_semaphore() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }

    // セマフォ版（並列）
    let mut semaphore_a = Grid::new();
    let mut semaphore_b = Grid::new();

    jacobi_steps_parallel_counter(&mut semaphore_a, &mut semaphore_b, TEST_STEPS);

    // TEST_STEPSが偶数の場合、最終結果はgrid_aに、奇数の場合はgrid_bにある
    let final_single = if TEST_STEPS.is_multiple_of(2) {
        &single_a
    } else {
        &single_b
    };

    // セマフォ版も同様にswapを考慮
    let final_semaphore = if TEST_STEPS.is_multiple_of(2) {
        &semaphore_a
    } else {
        &semaphore_b
    };

    assert!(
        grids_are_equal(final_single, final_semaphore),
        "Single-thread and semaphore implementations produce different results"
    );

    println!("✓ Single vs Semaphore: Results match!");
}

#[test]
fn test_single_step_consistency() {
    // 同じ初期条件で2回実行して結果が同じか確認
    let mut grid1_a = Grid::new();
    let mut grid1_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&grid1_a, &mut grid1_b);
        std::mem::swap(&mut grid1_a, &mut grid1_b);
    }

    let mut grid2_a = Grid::new();
    let mut grid2_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&grid2_a, &mut grid2_b);
        std::mem::swap(&mut grid2_a, &mut grid2_b);
    }

    let final1 = if TEST_STEPS.is_multiple_of(2) { &grid1_a } else { &grid1_b };
    let final2 = if TEST_STEPS.is_multiple_of(2) { &grid2_a } else { &grid2_b };

    assert!(
        grids_are_equal(final1, final2),
        "Single-thread implementation is not deterministic"
    );

    println!("✓ Single-thread consistency: Results match!");
}

#[test]
fn test_heat_source_preserved() {
    use jacobi_rust::grid::{N, M};

    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    // 複数ステップ実行
    for _ in 0..TEST_STEPS {
        jacobi_step(&grid_a, &mut grid_b);
        std::mem::swap(&mut grid_a, &mut grid_b);
    }

    let final_grid = if TEST_STEPS.is_multiple_of(2) { &grid_a } else { &grid_b };

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
    use jacobi_rust::grid::{N, M};

    let mut grid_a = Grid::new();
    let mut grid_b = Grid::new();

    // 複数ステップ実行
    for _ in 0..TEST_STEPS {
        jacobi_step(&grid_a, &mut grid_b);
        std::mem::swap(&mut grid_a, &mut grid_b);
    }

    let final_grid = if TEST_STEPS.is_multiple_of(2) { &grid_a } else { &grid_b };

    // 境界が0.0のまま保持されているか確認
    // 上境界 (i=0)
    for j in 0..M {
        let idx = j;
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Top boundary at (0, {}) should be 0.0, but got {}",
            j, final_grid.data[idx]
        );
    }

    // 下境界 (i=N-1)
    for j in 0..M {
        let idx = (N - 1) * M + j;
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Bottom boundary at ({}, {}) should be 0.0, but got {}",
            N - 1, j, final_grid.data[idx]
        );
    }

    // 左境界 (j=0)
    for i in 0..N {
        let idx = i * M;
        assert_eq!(
            final_grid.data[idx], 0.0,
            "Left boundary at ({}, 0) should be 0.0, but got {}",
            i, final_grid.data[idx]
        );
    }

    // 右境界 (j=M-1)
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

#[test]
fn test_single_vs_barrier_parallel() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }

    // バリア並列版
    let mut barrier_a = Grid::new();
    let mut barrier_b = Grid::new();

    barrier_parallel(&mut barrier_a, &mut barrier_b, TEST_STEPS);

    // TEST_STEPSが偶数の場合、最終結果はgrid_aに、奇数の場合はgrid_bにある
    let final_single = if TEST_STEPS.is_multiple_of(2) {
        &single_a
    } else {
        &single_b
    };

    let final_barrier = if TEST_STEPS.is_multiple_of(2) {
        &barrier_a
    } else {
        &barrier_b
    };

    assert!(
        grids_are_equal(final_single, final_barrier),
        "Single-thread and barrier parallel implementations produce different results"
    );

    println!("✓ Single vs Barrier Parallel: Results match!");
}

#[test]
fn test_single_vs_barrier_parallel_02() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }

    // バリア並列版02（境界共有版）
    let mut barrier_a = Grid::new();
    let mut barrier_b = Grid::new();

    barrier_parallel_02(&mut barrier_a, &mut barrier_b, TEST_STEPS);

    // TEST_STEPSが偶数の場合、最終結果はgrid_aに、奇数の場合はgrid_bにある
    let final_single = if TEST_STEPS.is_multiple_of(2) {
        &single_a
    } else {
        &single_b
    };

    let final_barrier = if TEST_STEPS.is_multiple_of(2) {
        &barrier_a
    } else {
        &barrier_b
    };

    assert!(
        grids_are_equal(final_single, final_barrier),
        "Single-thread and barrier parallel 02 implementations produce different results"
    );

    println!("✓ Single vs Barrier Parallel 02: Results match!");
}

#[test]
fn test_barrier_parallel_vs_barrier_parallel_02() {
    // バリア並列版
    let mut barrier_a = Grid::new();
    let mut barrier_b = Grid::new();

    barrier_parallel(&mut barrier_a, &mut barrier_b, TEST_STEPS);

    // バリア並列版02（境界共有版）
    let mut barrier02_a = Grid::new();
    let mut barrier02_b = Grid::new();

    barrier_parallel_02(&mut barrier02_a, &mut barrier02_b, TEST_STEPS);

    // TEST_STEPSが偶数の場合、最終結果はgrid_aに、奇数の場合はgrid_bにある
    let final_barrier = if TEST_STEPS.is_multiple_of(2) {
        &barrier_a
    } else {
        &barrier_b
    };

    let final_barrier02 = if TEST_STEPS.is_multiple_of(2) {
        &barrier02_a
    } else {
        &barrier02_b
    };

    assert!(
        grids_are_equal(final_barrier, final_barrier02),
        "Barrier parallel and barrier parallel 02 implementations produce different results"
    );

    println!("✓ Barrier Parallel vs Barrier Parallel 02: Results match!");
}

#[test]
fn test_single_vs_rayon() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }

    // Rayon並列版
    let mut rayon_a = Grid::new();
    let mut rayon_b = Grid::new();

    rayon_parallel(&mut rayon_a, &mut rayon_b, TEST_STEPS);

    let final_single = if TEST_STEPS.is_multiple_of(2) {
        &single_a
    } else {
        &single_b
    };

    assert!(
        grids_are_equal(final_single, &rayon_a),
        "Single-thread and rayon implementations produce different results"
    );

    println!("✓ Single vs Rayon: Results match!");
}

#[test]
fn test_single_vs_rayon_v2() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }

    // Rayon並列版 v2
    let mut rayon_a = Grid::new();
    let mut rayon_b = Grid::new();

    rayon_parallel_v2(&mut rayon_a, &mut rayon_b, TEST_STEPS);

    let final_single = if TEST_STEPS.is_multiple_of(2) {
        &single_a
    } else {
        &single_b
    };

    assert!(
        grids_are_equal(final_single, &rayon_a),
        "Single-thread and rayon v2 implementations produce different results"
    );

    println!("✓ Single vs Rayon v2: Results match!");
}

#[test]
fn test_single_vs_barrier_parallel_03() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();

    for _ in 0..TEST_STEPS {
        jacobi_step(&single_a, &mut single_b);
        std::mem::swap(&mut single_a, &mut single_b);
    }

    // バリア並列版03（バリア1回のみ）
    let mut barrier_a = Grid::new();
    let mut barrier_b = Grid::new();

    barrier_parallel_03(&mut barrier_a, &mut barrier_b, TEST_STEPS);

    let final_single = if TEST_STEPS.is_multiple_of(2) {
        &single_a
    } else {
        &single_b
    };

    let final_barrier = if TEST_STEPS.is_multiple_of(2) {
        &barrier_a
    } else {
        &barrier_b
    };

    assert!(
        grids_are_equal(final_single, final_barrier),
        "Single-thread and barrier parallel 03 implementations produce different results"
    );

    println!("✓ Single vs Barrier Parallel 03: Results match!");
}

#[test]
fn test_barrier_parallel_02_vs_03() {
    // バリア並列版02（バリア2回）
    let mut barrier02_a = Grid::new();
    let mut barrier02_b = Grid::new();

    barrier_parallel_02(&mut barrier02_a, &mut barrier02_b, TEST_STEPS);

    // バリア並列版03（バリア1回）
    let mut barrier03_a = Grid::new();
    let mut barrier03_b = Grid::new();

    barrier_parallel_03(&mut barrier03_a, &mut barrier03_b, TEST_STEPS);

    let final_barrier02 = if TEST_STEPS.is_multiple_of(2) {
        &barrier02_a
    } else {
        &barrier02_b
    };

    let final_barrier03 = if TEST_STEPS.is_multiple_of(2) {
        &barrier03_a
    } else {
        &barrier03_b
    };

    assert!(
        grids_are_equal(final_barrier02, final_barrier03),
        "Barrier parallel 02 and 03 implementations produce different results"
    );

    println!("✓ Barrier Parallel 02 vs 03: Results match!");
}
