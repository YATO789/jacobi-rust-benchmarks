# Jacobi Method Benchmark Results (By Category)

## 1. Single Thread (Sequential) Baseline

### Performance Comparison (Median Time in ms)

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| C Single Thread | 4.76 | 13.48 | 76.22 | 270.43 |
| Rust Single Thread | 6.14 | 14.52 | 77.55 | 309.32 |

### Language Overhead (Rust vs C)

| Grid Size | Overhead | Notes |
|:---|---:|:---|
| 64x64 | 1.29x | Small grid, overhead more visible |
| 128x128 | 1.08x | Overhead decreasing |
| 512x512 | 1.02x | Nearly equivalent |
| 1024x1024 | 1.14x | Slight overhead at large scale |

---

## 2. Semaphore Implementation

### Performance Comparison (Median Time in ms)

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| **C Semaphore** | 3.93 | 8.12 | 37.88 | 134.55 |
| **Rust Safe Semaphore** | 4.90 | 10.45 | 43.70 | 163.19 |
| **Rust Unsafe Semaphore** | 4.83 | 8.69 | 40.66 | 144.55 |

### Speedup vs Single Thread

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| C Semaphore | 1.21x | 1.66x | 2.01x | 2.01x |
| Rust Safe Semaphore | 1.25x | 1.39x | 1.77x | 1.90x |
| Rust Unsafe Semaphore | 1.27x | 1.67x | 1.91x | 2.14x |

### Safety Cost (Rust Safe / Rust Unsafe)

| Grid Size | Safety Cost | Overhead |
|:---|---:|---:|
| 64x64 | 1.01x | 1% |
| 128x128 | 1.20x | 20% |
| 512x512 | 1.07x | 7% |
| 1024x1024 | 1.13x | 13% |

### Language Comparison (Rust Safe / C)

| Grid Size | Overhead | Analysis |
|:---|---:|:---|
| 64x64 | 1.25x | Implementation strategy difference |
| 128x128 | 1.29x | Mutex overhead visible |
| 512x512 | 1.15x | Scaling better |
| 1024x1024 | 1.21x | Consistent overhead |

---

## 3. Barrier Implementation

### Performance Comparison (Median Time in ms)

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| **C Barrier** | 9.41 | 15.82 | 46.17 | 165.88 |
| **Rust Safe Barrier** | 12.77 | 18.92 | 49.66 | 174.56 |
| **Rust Unsafe Barrier** | 10.46 | 15.16 | 42.60 | 153.85 |

### Speedup vs Single Thread

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| C Barrier | 0.51x | 0.85x | 1.65x | 1.63x |
| Rust Safe Barrier | 0.48x | 0.77x | 1.56x | 1.77x |
| Rust Unsafe Barrier | 0.59x | 0.96x | 1.82x | 2.01x |

### Safety Cost (Rust Safe / Rust Unsafe)

| Grid Size | Safety Cost | Overhead |
|:---|---:|---:|
| 64x64 | 1.22x | 22% |
| 128x128 | 1.25x | 25% |
| 512x512 | 1.17x | 17% |
| 1024x1024 | 1.13x | 13% |

### Language Comparison (Rust Safe / C)

| Grid Size | Overhead | Analysis |
|:---|---:|:---|
| 64x64 | 1.36x | Higher overhead on small grids |
| 128x128 | 1.20x | Decreasing overhead |
| 512x512 | 1.08x | Nearly equivalent strategy |
| 1024x1024 | 1.05x | Very close performance |

---

## 4. OpenMP / Rayon Implementation

### Performance Comparison (Median Time in ms)

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| **C OpenMP** | 22.92 | 23.71 | 39.10 | 115.68 |
| **Rust Safe Rayon** | 13.50 | 19.22 | 48.00 | 173.64 |
| **Rust Unsafe Rayon** | 12.88 | 18.47 | 49.63 | 175.85 |

### Speedup vs Single Thread

| Implementation | 64x64 | 128x128 | 512x512 | 1024x1024 |
|:---|---:|---:|---:|---:|
| C OpenMP | 0.21x | 0.57x | 1.95x | 2.34x |
| Rust Safe Rayon | 0.45x | 0.76x | 1.62x | 1.78x |
| Rust Unsafe Rayon | 0.48x | 0.79x | 1.56x | 1.76x |

### Safety Cost (Rust Safe / Rust Unsafe)

| Grid Size | Safety Cost | Overhead |
|:---|---:|---:|
| 64x64 | 1.05x | 5% |
| 128x128 | 1.04x | 4% |
| 512x512 | 0.97x | -3% (Safe faster!) |
| 1024x1024 | 0.99x | -1% (Safe faster!) |

### Language Comparison (Rust Safe / C OpenMP)

| Grid Size | Rayon Advantage | Analysis |
|:---|---:|:---|
| 64x64 | 0.59x | Rayon 1.70x faster |
| 128x128 | 0.81x | Rayon 1.23x faster |
| 512x512 | 1.23x | OpenMP 1.23x faster |
| 1024x1024 | 1.50x | OpenMP 1.50x faster |

---

## Summary

### Best Performers by Grid Size

| Grid Size | Best C | Best Rust Safe | Best Rust Unsafe | Overall Winner |
|:---|:---|:---|:---|:---|
| 64x64 | Semaphore (3.93ms) | Semaphore (4.90ms) | Semaphore (4.83ms) | **C Semaphore** |
| 128x128 | Semaphore (8.12ms) | Semaphore (10.45ms) | Semaphore (8.69ms) | **C Semaphore** |
| 512x512 | Semaphore (37.88ms) | OpenMP (39.10ms) | Barrier Unsafe (42.60ms) | **C Semaphore** |
| 1024x1024 | OpenMP (115.68ms) | Semaphore (163.19ms) | Semaphore (144.55ms) | **C OpenMP** |

### Key Observations

1. **Semaphore**: Most consistent performer, especially for medium-large grids
2. **Barrier**: Moderate overhead, better at larger grids
3. **OpenMP/Rayon**: 
   - High overhead on small grids
   - Best scaling at 1024x1024 (C OpenMP: 2.34x speedup)
   - Rayon better on small grids, OpenMP better on large grids
4. **Safety Cost**: Generally 1-25%, lowest in Rayon (sometimes negative!)
5. **Language Overhead**: Rust 2-36% slower than C for equivalent implementations
