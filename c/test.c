#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#include "common/jacobi_common.h"
#include "semaphore/jacobi_semaphore.h"
#include "barrier/jacobi_barrier.h"
#include "omp/jacobi_omp.h"
#include "naive/jacobi_naive.h"

#define TEST_STEPS 10
#define EPSILON 1e-10

// Test statistics
static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;

// Single thread reference implementation
void jacobi_step_single(Grid *a, Grid *b, int steps) {
    double factor = ALPHA * DT / (DX * DX);
    double *ptr_a = a->data;
    double *ptr_b = b->data;

    for (int t = 0; t < steps; t++) {
        for (int i = 1; i < N - 1; i++) {
            for (int j = 1; j < M - 1; j++) {
                int idx = i * M + j;
                double laplacian = ptr_a[(i + 1) * M + j] + ptr_a[(i - 1) * M + j] +
                                   ptr_a[i * M + (j + 1)] + ptr_a[i * M + (j - 1)] -
                                   4.0 * ptr_a[idx];
                ptr_b[idx] = ptr_a[idx] + factor * laplacian;
            }
        }

        ptr_b[(N / 2) * M + (M / 2)] = 100.0;

        double *temp = ptr_a;
        ptr_a = ptr_b;
        ptr_b = temp;
    }

    if (steps % 2 == 1) {
        memcpy(a->data, ptr_a, N * M * sizeof(double));
    }
}

// Get final grid based on step count
Grid *get_final_grid(Grid *a, Grid *b, int steps) {
    return (steps % 2 == 0) ? a : b;
}

// Compare two grids
int grids_are_equal(const Grid *grid1, const Grid *grid2) {
    for (int i = 0; i < N * M; i++) {
        double diff = fabs(grid1->data[i] - grid2->data[i]);
        if (diff > EPSILON) {
            fprintf(stderr, "  Mismatch at index %d: %.10e vs %.10e (diff: %.10e)\n",
                    i, grid1->data[i], grid2->data[i], diff);
            return 0;
        }
    }
    return 1;
}

// Test runner macro
#define RUN_TEST(test_name, test_func) do { \
    printf("Running %s...\n", test_name); \
    tests_run++; \
    if (test_func()) { \
        printf("  ✓ PASSED\n\n"); \
        tests_passed++; \
    } else { \
        printf("  ✗ FAILED\n\n"); \
        tests_failed++; \
    } \
} while(0)


// Test: Single vs Safe Semaphore
int test_single_vs_safe_semaphore(void) {
    Grid single_a, single_b;
    grid_init(&single_a);
    grid_init(&single_b);
    jacobi_step_single(&single_a, &single_b, TEST_STEPS);

    Grid safe_sem_a, safe_sem_b;
    grid_init(&safe_sem_a);
    grid_init(&safe_sem_b);
    run_safe_semaphore_optimized(&safe_sem_a, &safe_sem_b, TEST_STEPS);

    Grid *final_single = get_final_grid(&single_a, &single_b, TEST_STEPS);
    Grid *final_safe_sem = get_final_grid(&safe_sem_a, &safe_sem_b, TEST_STEPS);

    int result = grids_are_equal(final_single, final_safe_sem);

    grid_free(&single_a);
    grid_free(&single_b);
    grid_free(&safe_sem_a);
    grid_free(&safe_sem_b);

    return result;
}

// Test: Single vs Barrier
int test_single_vs_barrier(void) {
    Grid single_a, single_b;
    grid_init(&single_a);
    grid_init(&single_b);
    jacobi_step_single(&single_a, &single_b, TEST_STEPS);

    Grid barrier_a, barrier_b;
    grid_init(&barrier_a);
    grid_init(&barrier_b);
    jacobi_step_barrier(&barrier_a, &barrier_b, TEST_STEPS);

    Grid *final_single = get_final_grid(&single_a, &single_b, TEST_STEPS);
    Grid *final_barrier = get_final_grid(&barrier_a, &barrier_b, TEST_STEPS);

    int result = grids_are_equal(final_single, final_barrier);

    grid_free(&single_a);
    grid_free(&single_b);
    grid_free(&barrier_a);
    grid_free(&barrier_b);

    return result;
}

// Test: Single vs OpenMP
int test_single_vs_omp(void) {
    Grid single_a, single_b;
    grid_init(&single_a);
    grid_init(&single_b);
    jacobi_step_single(&single_a, &single_b, TEST_STEPS);

    Grid omp_a, omp_b;
    grid_init(&omp_a);
    grid_init(&omp_b);
    jacobi_step_omp(&omp_a, &omp_b, TEST_STEPS);

    Grid *final_single = get_final_grid(&single_a, &single_b, TEST_STEPS);
    Grid *final_omp = get_final_grid(&omp_a, &omp_b, TEST_STEPS);

    int result = grids_are_equal(final_single, final_omp);

    grid_free(&single_a);
    grid_free(&single_b);
    grid_free(&omp_a);
    grid_free(&omp_b);

    return result;
}

// Test: Single vs Naive
int test_single_vs_naive(void) {
    Grid single_a, single_b;
    grid_init(&single_a);
    grid_init(&single_b);
    jacobi_step_single(&single_a, &single_b, TEST_STEPS);

    Grid naive_a, naive_b;
    grid_init(&naive_a);
    grid_init(&naive_b);
    jacobi_step_naive(&naive_a, &naive_b, TEST_STEPS);

    Grid *final_single = get_final_grid(&single_a, &single_b, TEST_STEPS);
    Grid *final_naive = get_final_grid(&naive_a, &naive_b, TEST_STEPS);

    int result = grids_are_equal(final_single, final_naive);

    grid_free(&single_a);
    grid_free(&single_b);
    grid_free(&naive_a);
    grid_free(&naive_b);

    return result;
}


// Test: Single step consistency
int test_single_step_consistency(void) {
    Grid grid1_a, grid1_b;
    grid_init(&grid1_a);
    grid_init(&grid1_b);
    jacobi_step_single(&grid1_a, &grid1_b, TEST_STEPS);

    Grid grid2_a, grid2_b;
    grid_init(&grid2_a);
    grid_init(&grid2_b);
    jacobi_step_single(&grid2_a, &grid2_b, TEST_STEPS);

    Grid *final1 = get_final_grid(&grid1_a, &grid1_b, TEST_STEPS);
    Grid *final2 = get_final_grid(&grid2_a, &grid2_b, TEST_STEPS);

    int result = grids_are_equal(final1, final2);

    grid_free(&grid1_a);
    grid_free(&grid1_b);
    grid_free(&grid2_a);
    grid_free(&grid2_b);

    return result;
}

// Test: Heat source preserved
int test_heat_source_preserved(void) {
    Grid grid_a, grid_b;
    grid_init(&grid_a);
    grid_init(&grid_b);
    jacobi_step_single(&grid_a, &grid_b, TEST_STEPS);

    Grid *final_grid = get_final_grid(&grid_a, &grid_b, TEST_STEPS);

    int heat_source_idx = (N / 2) * M + (M / 2);
    int result = (fabs(final_grid->data[heat_source_idx] - 100.0) < EPSILON);

    if (!result) {
        fprintf(stderr, "  Heat source at (%d, %d) should be 100.0, but got %.10e\n",
                N / 2, M / 2, final_grid->data[heat_source_idx]);
    }

    grid_free(&grid_a);
    grid_free(&grid_b);

    return result;
}

// Test: Boundary conditions
int test_boundary_conditions(void) {
    Grid grid_a, grid_b;
    grid_init(&grid_a);
    grid_init(&grid_b);
    jacobi_step_single(&grid_a, &grid_b, TEST_STEPS);

    Grid *final_grid = get_final_grid(&grid_a, &grid_b, TEST_STEPS);

    int all_zero = 1;

    // Top boundary (i=0)
    for (int j = 0; j < M; j++) {
        if (fabs(final_grid->data[j]) > EPSILON) {
            fprintf(stderr, "  Top boundary at (0, %d) should be 0.0, but got %.10e\n",
                    j, final_grid->data[j]);
            all_zero = 0;
            break;
        }
    }

    // Bottom boundary (i=N-1)
    for (int j = 0; j < M && all_zero; j++) {
        int idx = (N - 1) * M + j;
        if (fabs(final_grid->data[idx]) > EPSILON) {
            fprintf(stderr, "  Bottom boundary at (%d, %d) should be 0.0, but got %.10e\n",
                    N - 1, j, final_grid->data[idx]);
            all_zero = 0;
            break;
        }
    }

    // Left boundary (j=0)
    for (int i = 0; i < N && all_zero; i++) {
        int idx = i * M;
        if (fabs(final_grid->data[idx]) > EPSILON) {
            fprintf(stderr, "  Left boundary at (%d, 0) should be 0.0, but got %.10e\n",
                    i, final_grid->data[idx]);
            all_zero = 0;
            break;
        }
    }

    // Right boundary (j=M-1)
    for (int i = 0; i < N && all_zero; i++) {
        int idx = i * M + (M - 1);
        if (fabs(final_grid->data[idx]) > EPSILON) {
            fprintf(stderr, "  Right boundary at (%d, %d) should be 0.0, but got %.10e\n",
                    i, M - 1, final_grid->data[idx]);
            all_zero = 0;
            break;
        }
    }

    grid_free(&grid_a);
    grid_free(&grid_b);

    return all_zero;
}

int main(void) {
    printf("=== Jacobi C Implementation Tests ===\n");
    printf("Grid size: %dx%d, Test steps: %d\n\n", N, M, TEST_STEPS);

    RUN_TEST("test_single_vs_safe_semaphore", test_single_vs_safe_semaphore);
    RUN_TEST("test_single_vs_barrier", test_single_vs_barrier);
    RUN_TEST("test_single_vs_omp", test_single_vs_omp);
    RUN_TEST("test_single_vs_naive", test_single_vs_naive);
    RUN_TEST("test_single_step_consistency", test_single_step_consistency);
    RUN_TEST("test_heat_source_preserved", test_heat_source_preserved);
    RUN_TEST("test_boundary_conditions", test_boundary_conditions);

    printf("=== Test Summary ===\n");
    printf("Total:  %d\n", tests_run);
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);

    return (tests_failed == 0) ? 0 : 1;
}
