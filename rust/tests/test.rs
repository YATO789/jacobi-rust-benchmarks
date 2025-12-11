use jacobi_rust::grid::{Grid, N, M};

// main.rsで使用されているすべての実装をインポート
use jacobi_rust::implementations::safe::single::jacobi_step;
use jacobi_rust::implementations::unsafe_impl::unsafe_atomic_counter::unsafe_atomic_counter;
use jacobi_rust::implementations::safe::barrier::barrier_parallel::barrier_parallel;
use jacobi_rust::implementations::safe::rayon::rayon::rayon_parallel;
use jacobi_rust::implementations::safe::atomic_counter::atomic_counter::atomic_counter;
use jacobi_rust::implementations::unsafe_impl::rayon_unsafe::rayon_unsafe;
use jacobi_rust::implementations::unsafe_impl::single_unsafe::jacobi_step_unsafe;

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
fn test_single_vs_unsafe_atomic_counter() {
    // シングルスレッド版 (正解データ)
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Unsafe Atomic Counter版
    let mut counter_a = Grid::new();
    let mut counter_b = Grid::new();
    unsafe_atomic_counter(&mut counter_a, &mut counter_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_counter = get_final_grid(&counter_a, &counter_b);

    assert!(
        grids_are_equal(final_single, final_counter),
        "Single-thread and Unsafe Atomic Counter implementations produce different results"
    );

    println!("✓ Single vs Unsafe Atomic Counter: Results match!");
}

#[test]
fn test_single_vs_safe_atomic_counter() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Safe Atomic Counter版
    let mut counter_a = Grid::new();
    let mut counter_b = Grid::new();
    atomic_counter(&mut counter_a, &mut counter_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_counter = get_final_grid(&counter_a, &counter_b);

    assert!(
        grids_are_equal(final_single, final_counter),
        "Single-thread and Safe Atomic Counter implementations produce different results"
    );

    println!("✓ Single vs Safe Atomic Counter: Results match!");
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
fn test_single_vs_rayon_unsafe() {
    // シングルスレッド版
    let mut single_a = Grid::new();
    let mut single_b = Grid::new();
    jacobi_step(&mut single_a, &mut single_b, TEST_STEPS);

    // Rayon Unsafe版
    let mut rayon_unsafe_a = Grid::new();
    let mut rayon_unsafe_b = Grid::new();
    rayon_unsafe(&mut rayon_unsafe_a, &mut rayon_unsafe_b, TEST_STEPS);

    let final_single = get_final_grid(&single_a, &single_b);
    let final_rayon_unsafe = get_final_grid(&rayon_unsafe_a, &rayon_unsafe_b);

    assert!(
        grids_are_equal(final_single, final_rayon_unsafe),
        "Single-thread and Rayon Unsafe implementations produce different results"
    );

    println!("✓ Single vs Rayon Unsafe: Results match!");
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

#[test]
fn test_single_safe_vs_unsafe() {
    // Safe版シングルスレッド
    let mut safe_a = Grid::new();
    let mut safe_b = Grid::new();
    jacobi_step(&mut safe_a, &mut safe_b, TEST_STEPS);

    // Unsafe版シングルスレッド
    let mut unsafe_a = Grid::new();
    let mut unsafe_b = Grid::new();
    jacobi_step_unsafe(&mut unsafe_a, &mut unsafe_b, TEST_STEPS);

    let final_safe = get_final_grid(&safe_a, &safe_b);
    let final_unsafe = get_final_grid(&unsafe_a, &unsafe_b);

    assert!(
        grids_are_equal(final_safe, final_unsafe),
        "Safe and Unsafe single-thread implementations produce different results"
    );

    println!("✓ Single Safe vs Unsafe: Results match!");
}