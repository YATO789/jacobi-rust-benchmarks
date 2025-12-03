# Jacobi Method Benchmark Results

## Performance Comparison Table (Median Time in ms)

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| C Single Thread | 4.76 | 13.48 | 76.22 | 270.43 |
| C Safe Semaphore | 3.93 | 8.12 | 37.88 | 134.55 |
| C Barrier | 9.41 | 15.82 | 46.17 | 165.88 |
| C OpenMP | 22.92 | 23.71 | 39.10 | 115.68 |
| Rust Single Thread | 6.14 | 14.52 | 77.55 | 309.32 |
| Rust Unsafe Semaphore | 4.83 | 8.69 | 40.66 | 144.55 |
| Rust Safe Semaphore | 4.90 | 10.45 | 43.70 | 163.19 |
| Rust Barrier | 12.77 | 18.92 | 49.66 | 174.56 |
| Rust Barrier Unsafe | 10.46 | 15.16 | 42.60 | 153.85 |
| Rust Rayon | 13.50 | 19.22 | 48.00 | 173.64 |
| Rust Rayon Unsafe | 12.88 | 18.47 | 49.63 | 175.85 |

## Performance Improvement (vs Single Thread)

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| C Safe Semaphore | 1.21x | 1.66x | 2.01x | 2.01x |
| C Barrier | 0.51x | 0.85x | 1.65x | 1.63x |
| C OpenMP | 0.21x | 0.57x | 1.95x | 2.34x |
| Rust Unsafe Semaphore | 1.27x | 1.67x | 1.91x | 2.14x |
| Rust Safe Semaphore | 1.25x | 1.39x | 1.77x | 1.90x |
| Rust Barrier | 0.48x | 0.77x | 1.56x | 1.77x |
| Rust Barrier Unsafe | 0.59x | 0.96x | 1.82x | 2.01x |
| Rust Rayon | 0.45x | 0.76x | 1.62x | 1.78x |
| Rust Rayon Unsafe | 0.48x | 0.79x | 1.56x | 1.76x |