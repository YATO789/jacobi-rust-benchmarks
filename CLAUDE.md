# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains benchmark implementations of the Jacobi method for solving 2D heat equations in both **Rust** and **C**. The implementations compare various parallel strategies including semaphores, barriers, OpenMP, Rayon, and unsafe optimizations.

## Build and Run Commands

### Integrated Benchmark Script (Recommended)

The `scripts/run_benchmark.sh` script runs fair comparisons between Rust and C implementations:

```bash
# Run with default settings (1024×1024 grid, 1000 steps)
./scripts/run_benchmark.sh

# Run with custom parameters
./scripts/run_benchmark.sh -n 512 -s 500    # 512×512 grid, 500 steps
./scripts/run_benchmark.sh --grid-size 2048 --steps 2000

# Change measurement iterations
./scripts/run_benchmark.sh -i 20 -w 5       # 20 iterations, 5 warmup runs
```

This script automatically:
1. Updates parameter files (`rust/src/grid.rs` and `c/common/jacobi_common.h`)
2. Builds both Rust and C implementations
3. Runs benchmarks with cooldown periods
4. Saves results to `scripts/benchmark_results/`

### Test Correctness

Verify that Rust and C implementations produce identical results:

```bash
./scripts/test_correctness.sh
```

This compares all 6 implementations (Single, Unsafe Semaphore, Safe Semaphore, Barrier, OpenMP/Rayon, Unsafe Optimized) at binary level.

### Individual Build and Run

#### Rust
```bash
cd rust
cargo build --release      # Build
cargo run --release        # Run benchmarks (default 2 threads)
cargo run --release -- 4   # Run with 4 threads (affects Rayon only)
cargo test                 # Run tests
```

#### C
```bash
cd c
make                       # Build
./jacobi_bench             # Run benchmarks (default 2 threads)
./jacobi_bench 4           # Run with 4 threads (affects OpenMP only)
make test                  # Run tests
make clean                 # Clean build artifacts
```

**Important Notes:**
- On macOS, OpenMP requires `libomp` installed via Homebrew: `brew install libomp`
- The Makefile automatically detects macOS and configures OpenMP accordingly
- Thread count argument only affects OpenMP (C) and Rayon (Rust); other implementations always use 2 threads

## Benchmark Configuration

All implementations share these parameters (defined in `rust/src/grid.rs` and `c/common/jacobi_common.h`):
- Grid size: 1024×1024 (N × M)
- Time steps: 1000 iterations
- Warmup steps: 10 iterations
- Benchmark iterations: Configurable in main.rs/main.c (default 10)
- Heat source: Fixed at grid center (N/2, M/2) at 100.0
- Cache line alignment: 64 bytes (using aligned-vec in Rust, manual alignment in C)

## Architecture Overview

### Rust Implementation Structure

The Rust code follows a modular architecture:

```
rust/src/
├── lib.rs                  # Library root
├── main.rs                 # Benchmark harness (runs all implementations)
├── grid.rs                 # Grid struct with aligned-vec, constants
├── bin/
│   └── test_output.rs      # Utility to output grid data
├── tests/
│   └── test.rs             # Correctness tests (9 test cases)
└── implementations/
    ├── mod.rs
    ├── safe/               # Safe Rust implementations
    │   ├── single.rs       # Single-threaded baseline
    │   ├── barrier/        # Barrier-based parallelization
    │   │   └── barrier_parallel.rs
    │   ├── semaphore/      # Semaphore-based parallelization
    │   │   └── semaphore_optimized.rs
    │   └── rayon/          # Rayon parallel iterator
    │       └── rayon.rs
    └── unsafe_impl/        # Unsafe implementations for performance
        ├── unsafe_semaphore.rs   # Atomic counter-based synchronization
        ├── barrier_unsafe.rs     # Barrier with unsafe optimizations
        ├── rayon_unsafe.rs       # Rayon with unsafe optimizations
        └── single_unsafe.rs      # Unsafe single-threaded baseline
```

**Key Architecture Patterns:**
- **Grid structure**: Uses `aligned-vec` crate for 64-byte cache line alignment
- **Grid splitting**: Parallel implementations divide the grid horizontally into two halves (upper/lower)
- **Boundary sharing**: Threads share boundary rows using Mutex-protected buffers
- **Double buffering**: All implementations use src/dst buffer swapping to avoid conflicts
- **5-point stencil**: Heat diffusion uses standard Laplacian stencil (up, down, left, right, center)

### C Implementation Structure

```
c/
├── main.c                      # Benchmark harness
├── test.c                      # Correctness tests
├── test_output.c               # Utility to output grid data
├── Makefile                    # Build configuration (macOS OpenMP detection)
└── [implementation dirs]/
    ├── common/                 # Shared utilities
    │   ├── jacobi_common.h     # Constants and Grid struct
    │   └── jacobi_common.c     # Grid init/save/load functions
    ├── semaphore/              # Safe semaphore implementation
    │   ├── jacobi_semaphore.h
    │   └── jacobi_semaphore.c
    ├── barrier/                # Barrier implementation
    │   ├── jacobi_barrier.h
    │   └── jacobi_barrier.c
    ├── omp/                    # OpenMP parallel implementation
    │   ├── jacobi_omp.h
    │   └── jacobi_omp.c
    └── naive/                  # Naive parallel implementation
        ├── jacobi_naive.h
        └── jacobi_naive.c
```

### Parallel Strategy Comparison

The project implements 6 different strategies to compare performance and safety tradeoffs:

1. **Single Thread**: Sequential baseline for correctness verification
2. **Unsafe Semaphore**: Atomic counter-based synchronization with unsafe operations
3. **Safe Semaphore**: Semaphore-based synchronization with safe Rust abstractions
4. **Barrier**: Two-phase synchronization using barriers (detailed in logic/barrier_parallel_logic.md)
5. **OpenMP/Rayon**: High-level parallel libraries (OpenMP for C, Rayon for Rust)
6. **Unsafe Optimized**: Maximum performance with unsafe memory operations

## Important Implementation Details

### Boundary Synchronization Logic

The barrier implementation (documented in `logic/barrier_parallel_logic.md`) uses a two-phase approach:
1. **Phase 1**: Each thread writes its boundary row to a shared Mutex buffer
2. **Barrier sync**: Ensure both threads have written boundaries
3. **Phase 2**: Calculate internal rows (lock-free), then boundary rows (reading from shared buffers)
4. **Barrier sync**: Ensure all calculations complete before next iteration

This minimizes lock contention: ~75% of calculations are lock-free, only ~16% require Mutex access.

### Index Calculations

Grid is stored as a flat `Vec<f64>` (Rust) / `double*` (C) in row-major order:
- Index formula: `idx = i * M + j` where `i` is row, `j` is column
- 5-point stencil accesses: `idx-M` (up), `idx+M` (down), `idx-1` (left), `idx+1` (right), `idx` (center)
- Example from `rust/src/implementations/safe/single.rs`:
  ```rust
  let laplacian = src[idx - M] + src[idx + M] + src[idx - 1] + src[idx + 1] - 4.0 * src[idx];
  dst[idx] = src[idx] + DT * ALPHA / (DX * DX) * laplacian;
  ```

### Double Buffering Pattern

All implementations use double buffering to avoid read-write conflicts:
```rust
for step in 0..steps {
    // Calculate from src, write to dst
    compute(src, dst);
    std::mem::swap(&mut src, &mut dst);
}
// Result is in src if steps is even, dst if odd
```

### Grid Data Structure

**Rust** (`rust/src/grid.rs`):
- Uses `AVec<f64, ConstAlign<64>>` from `aligned-vec` crate for cache alignment
- Automatically aligns data to 64-byte boundaries
- Provides `save_to_file()` and `load_from_file()` for binary serialization

**C** (`c/common/jacobi_common.h`):
- Uses `aligned_alloc(CACHE_LINE_SIZE, size)` for manual alignment
- Must manually free with `free()` in `grid_free()`

### Testing Infrastructure

**Rust tests** (`rust/tests/test.rs`):
- 9 test cases comparing all implementations against single-threaded baseline
- Uses epsilon comparison (1e-10) for floating-point equality
- Tests: correctness, determinism, heat source preservation, boundary conditions

**C tests** (`c/test.c`):
- Mirrors Rust test structure
- Uses binary file comparison with Rust outputs

**Cross-validation** (`scripts/test_correctness.sh`):
- Verifies Rust and C produce identical binary outputs
- Tests all 6 implementations at 64×64 grid, 100 steps

## Modifying Parameters

When changing grid size or time steps, you must update both languages:

**Option 1: Use the benchmark script** (recommended)
```bash
./scripts/run_benchmark.sh -n 2048 -s 2000
```
This automatically updates both `rust/src/grid.rs` and `c/common/jacobi_common.h`.

**Option 2: Manual updates**
Edit both files to keep parameters synchronized:
- `rust/src/grid.rs`: `pub const N`, `pub const M`, `pub const TIME_STEPS`
- `c/common/jacobi_common.h`: `#define N`, `#define M`, `#define TIME_STEPS`

Then rebuild both implementations:
```bash
cd rust && cargo build --release
cd ../c && make clean && make
```

## Common Development Tasks

When modifying implementations:
- **Always maintain consistency** between Rust and C parameter definitions
- **Test with both single-threaded and parallel versions** to catch synchronization bugs
- **Verify heat source remains fixed** at grid center (N/2, M/2) = 100.0 after modifications
- **Run correctness tests** with `./scripts/test_correctness.sh` before benchmarking
- **Use release builds** for accurate performance measurements (`cargo build --release`, `make`)
- **Check resource cleanup** to avoid thread/mutex leaks (especially in C implementations)
