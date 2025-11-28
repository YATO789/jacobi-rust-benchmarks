use jacobi_rust::grid::Grid;
use jacobi_rust::implementations::single::jacobi_step;
use jacobi_rust::implementations::semaphore::jacobi_steps_parallel_counter;

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
    let final_single = if TEST_STEPS % 2 == 0 {
        &single_a
    } else {
        &single_b
    };

    // セマフォ版も同様にswapを考慮
    let final_semaphore = if TEST_STEPS % 2 == 0 {
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

    let final1 = if TEST_STEPS % 2 == 0 { &grid1_a } else { &grid1_b };
    let final2 = if TEST_STEPS % 2 == 0 { &grid2_a } else { &grid2_b };

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

    let final_grid = if TEST_STEPS % 2 == 0 { &grid_a } else { &grid_b };

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

    let final_grid = if TEST_STEPS % 2 == 0 { &grid_a } else { &grid_b };

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
