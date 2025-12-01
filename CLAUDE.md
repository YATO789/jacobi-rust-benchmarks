# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains benchmark implementations of the Jacobi method for solving 2D heat equations in both **Rust** and **C**. The implementations compare various parallel strategies including semaphores, barriers, channels, OpenMP, Rayon, and unsafe optimizations.

## Build and Run Commands

### Rust
```bash
# Build the project
cd rust && cargo build --release

# Run benchmarks
cargo run --release

# Run tests
cargo test
```

### C
```bash
# Build the C benchmarks
cd c && make

# Run C benchmarks
make run

# Clean build artifacts
make clean
```

**Important Notes:**
- On macOS, OpenMP requires `libomp` installed via Homebrew: `brew install libomp`
- The Makefile automatically detects macOS and configures OpenMP accordingly
- All benchmarks use 2 threads for fair comparison

## Benchmark Configuration

All implementations share these parameters (defined in `rust/src/grid.rs` and `c/jacobi_common.h`):
- Grid size: 1000x1000 (N × M)
- Time steps: 100 iterations
- Warmup steps: 10 iterations
- Benchmark iterations: 15 runs with 3 warmup runs
- Heat source: Fixed at grid center (N/2, M/2) at 100.0

## Architecture Overview

### Rust Implementation Structure

The Rust code follows a modular architecture:

```
rust/src/
├── lib.rs                  # Library root
├── main.rs                 # Benchmark harness
├── grid.rs                 # Grid struct and constants
└── implementations/
    ├── mod.rs
    ├── safe/               # Safe Rust implementations
    │   ├── single.rs       # Single-threaded baseline
    │   ├── barrier/        # Barrier-based parallelization
    │   ├── semaphore/      # Semaphore-based parallelization
    │   ├── channel/        # Channel-based parallelization
    │   └── rayon/          # Rayon parallel iterator
    └── unsafe/             # Unsafe implementations for comparison
        ├── unsafe_semaphore.rs
        └── parallel_unsafe.rs
```

**Key Architecture Patterns:**
- **Grid splitting**: Implementations divide the 1000x1000 grid horizontally into two halves (upper/lower)
- **Boundary sharing**: Threads share boundary rows using Mutex-protected buffers
- **Double buffering**: All implementations use src/dst buffer swapping to avoid conflicts
- **5-point stencil**: Heat diffusion uses a standard Laplacian stencil calculation

### C Implementation Structure

```
c/
├── main.c                  # Benchmark harness
├── jacobi_common.{c,h}     # Shared grid initialization
├── jacobi_semaphore.{c,h}  # Semaphore implementation
├── jacobi_barrier.{c,h}    # Barrier implementation
├── jacobi_omp.{c,h}        # OpenMP implementation
└── jacobi_naive.{c,h}      # Naive parallelization
```

### Parallel Strategy Comparison

1. **Barrier**: Split grid in half, use Barrier for phase synchronization, share boundary rows via Mutex
2. **Semaphore**: Row-level synchronization allowing fine-grained control of data dependencies
3. **Channel**: Message-passing between threads for boundary data exchange
4. **Rayon**: Data-parallel iteration over grid chunks using work-stealing
5. **OpenMP** (C only): Compiler directives for automatic parallelization
6. **Unsafe**: Manual memory management and synchronization for maximum performance

## Important Implementation Details

### Boundary Synchronization Logic

The barrier implementation (documented in `logic/barrier_parallel_logic.md`) uses a two-phase approach:
1. **Phase 1**: Each thread writes its boundary row to a shared Mutex buffer
2. **Barrier sync**: Ensure both threads have written boundaries
3. **Phase 2**: Calculate internal rows (lock-free), then boundary rows (reading from shared buffers)
4. **Barrier sync**: Ensure all calculations complete before next iteration

This minimizes lock contention: ~75% of calculations are lock-free, only ~16% require Mutex access.

### Index Calculations

Grid is stored as a flat `Vec<f64>` / `double*` in row-major order:
- Index formula: `idx = i * M + j` where `i` is row, `j` is column
- 5-point stencil accesses: `idx-M` (up), `idx+M` (down), `idx-1` (left), `idx+1` (right)

### Double Buffering Pattern

All implementations use:
```rust
for step in 0..steps {
    // Calculate from src, write to dst
    compute(src, dst);
    std::mem::swap(&mut src, &mut dst);
}
// Result is in src if steps is even, dst if odd
```

## Common Development Tasks

When modifying implementations:
- Maintain consistency between Rust and C parameter definitions
- Test with both single-threaded and parallel versions to catch synchronization bugs
- Verify heat source remains fixed at grid center after modifications
- Profile using `cargo build --release` for accurate performance measurements
- Check for proper cleanup of threads/mutexes to avoid resource leaks
