## 7. Results

### Benchmark Results (64x64 Grid, 1000 Steps)

#### Comparison by Median (Unit: ms)

| Implementation | C | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 4.757 | 6.136 | - |
| **Semaphore** | 3.934 | 4.899 | 4.834 |
| **Barrier** | 9.409 | 12.769 | 10.456 |
| **OpenMP/Rayon** | 22.921 | 13.505 | 12.881 |

#### Key Findings

**1. C vs Rust Safe Semaphore: C is 1.25x faster**
- C Semaphore: 3.934 ms
- Rust Safe Semaphore: 4.899 ms
- Reasons:
  - Boundary buffer copy overhead
  - Mutex lock/unlock cost
  - Implementation strategy differences

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafe is 1.01x faster**
- Safe Semaphore: 4.899 ms
- Unsafe Semaphore: 4.834 ms
- Reasons:
  - Elimination of bounds checking
  - No boundary buffer needed (direct memory access)

**3. Barrier: Safe vs Unsafe - Unsafe is 1.22x faster**
- Rust Safe Barrier: 12.769 ms
- Rust Unsafe Barrier: 10.456 ms
- Simple synchronization pattern with lower overhead

**4. Rayon: Balanced performance - Unsafe is 1.05x faster**
- Rust Safe Rayon: 13.505 ms
- Rust Unsafe Rayon: 12.881 ms
- Practical performance despite high-level abstraction

**5. OpenMP vs Rayon: OpenMP is 0.59x faster**
- C OpenMP: 22.921 ms
- Rust Safe Rayon: 13.505 ms

### Performance Ratio Analysis

#### Safety Cost
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| Implementation | Safety Cost | Notes |
|------|-------------|------|
| Semaphore | 1.01x | Boundary buffer + Mutex (1% overhead) |
| Barrier | 1.22x | Boundary buffer + Mutex (22% overhead) |
| Rayon | 1.05x | Bounds checking (5% overhead) |

→ **Safety cost is 1-22%**

#### Cross-Language Comparison
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| Implementation | Overhead | Analysis |
|------|----------|------|
| Semaphore | 1.25x | Implementation strategy difference is dominant |
| Barrier | 1.36x | Equivalent implementation strategy |
| Single Thread | 1.29x | Baseline comparison |

#### Parallel Speedup (vs Single Thread)

| Implementation | C Speedup | Rust Safe Speedup | Rust Unsafe Speedup |
|------|-----------|-------------------|---------------------|
| Semaphore | 1.21x | 1.25x | 1.27x |
| Barrier | 0.51x | 0.48x | 0.59x |
| OpenMP/Rayon | 0.21x | 0.45x | 0.48x |

---

### Benchmark Results (128x128 Grid, 1000 Steps)

#### Comparison by Median (Unit: ms)

| Implementation | C | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 13.476 | 14.522 | - |
| **Semaphore** | 8.117 | 10.450 | 8.694 |
| **Barrier** | 15.821 | 18.916 | 15.158 |
| **OpenMP/Rayon** | 23.707 | 19.223 | 18.469 |

#### Key Findings

**1. C vs Rust Safe Semaphore: C is 1.29x faster**
- C Semaphore: 8.117 ms
- Rust Safe Semaphore: 10.450 ms
- Reasons:
  - Boundary buffer copy overhead
  - Mutex lock/unlock cost
  - Implementation strategy differences

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafe is 1.20x faster**
- Safe Semaphore: 10.450 ms
- Unsafe Semaphore: 8.694 ms
- Reasons:
  - Elimination of bounds checking
  - No boundary buffer needed (direct memory access)

**3. Barrier: Safe vs Unsafe - Unsafe is 1.25x faster**
- Rust Safe Barrier: 18.916 ms
- Rust Unsafe Barrier: 15.158 ms
- Simple synchronization pattern with lower overhead

**4. Rayon: Balanced performance - Unsafe is 1.04x faster**
- Rust Safe Rayon: 19.223 ms
- Rust Unsafe Rayon: 18.469 ms
- Practical performance despite high-level abstraction

**5. OpenMP vs Rayon: OpenMP is 0.81x faster**
- C OpenMP: 23.707 ms
- Rust Safe Rayon: 19.223 ms

### Performance Ratio Analysis

#### Safety Cost
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| Implementation | Safety Cost | Notes |
|------|-------------|------|
| Semaphore | 1.20x | Boundary buffer + Mutex (20% overhead) |
| Barrier | 1.25x | Boundary buffer + Mutex (25% overhead) |
| Rayon | 1.04x | Bounds checking (4% overhead) |

→ **Safety cost is 4-25%**

#### Cross-Language Comparison
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| Implementation | Overhead | Analysis |
|------|----------|------|
| Semaphore | 1.29x | Implementation strategy difference is dominant |
| Barrier | 1.20x | Equivalent implementation strategy |
| Single Thread | 1.08x | Baseline comparison |

#### Parallel Speedup (vs Single Thread)

| Implementation | C Speedup | Rust Safe Speedup | Rust Unsafe Speedup |
|------|-----------|-------------------|---------------------|
| Semaphore | 1.66x | 1.39x | 1.67x |
| Barrier | 0.85x | 0.77x | 0.96x |
| OpenMP/Rayon | 0.57x | 0.76x | 0.79x |

---

### Benchmark Results (512x512 Grid, 1000 Steps)

#### Comparison by Median (Unit: ms)

| Implementation | C | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 76.215 | 77.546 | - |
| **Semaphore** | 37.877 | 43.697 | 40.656 |
| **Barrier** | 46.166 | 49.660 | 42.600 |
| **OpenMP/Rayon** | 39.097 | 48.003 | 49.633 |

#### Key Findings

**1. C vs Rust Safe Semaphore: C is 1.15x faster**
- C Semaphore: 37.877 ms
- Rust Safe Semaphore: 43.697 ms
- Reasons:
  - Boundary buffer copy overhead
  - Mutex lock/unlock cost
  - Implementation strategy differences

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafe is 1.07x faster**
- Safe Semaphore: 43.697 ms
- Unsafe Semaphore: 40.656 ms
- Reasons:
  - Elimination of bounds checking
  - No boundary buffer needed (direct memory access)

**3. Barrier: Safe vs Unsafe - Unsafe is 1.17x faster**
- Rust Safe Barrier: 49.660 ms
- Rust Unsafe Barrier: 42.600 ms
- Simple synchronization pattern with lower overhead

**4. Rayon: Balanced performance - Unsafe is 0.97x faster**
- Rust Safe Rayon: 48.003 ms
- Rust Unsafe Rayon: 49.633 ms
- Practical performance despite high-level abstraction

**5. OpenMP vs Rayon: OpenMP is 1.23x faster**
- C OpenMP: 39.097 ms
- Rust Safe Rayon: 48.003 ms

### Performance Ratio Analysis

#### Safety Cost
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| Implementation | Safety Cost | Notes |
|------|-------------|------|
| Semaphore | 1.07x | Boundary buffer + Mutex (7% overhead) |
| Barrier | 1.17x | Boundary buffer + Mutex (17% overhead) |
| Rayon | 0.97x | Bounds checking (-3% overhead) |

→ **Safety cost is -3-17%**

#### Cross-Language Comparison
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| Implementation | Overhead | Analysis |
|------|----------|------|
| Semaphore | 1.15x | Implementation strategy difference is dominant |
| Barrier | 1.08x | Equivalent implementation strategy |
| Single Thread | 1.02x | Baseline comparison |

#### Parallel Speedup (vs Single Thread)

| Implementation | C Speedup | Rust Safe Speedup | Rust Unsafe Speedup |
|------|-----------|-------------------|---------------------|
| Semaphore | 2.01x | 1.77x | 1.91x |
| Barrier | 1.65x | 1.56x | 1.82x |
| OpenMP/Rayon | 1.95x | 1.62x | 1.56x |

---

### Benchmark Results (1024x1024 Grid, 1000 Steps)

#### Comparison by Median (Unit: ms)

| Implementation | C | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 270.432 | 309.318 | - |
| **Semaphore** | 134.547 | 163.188 | 144.552 |
| **Barrier** | 165.879 | 174.559 | 153.852 |
| **OpenMP/Rayon** | 115.682 | 173.641 | 175.847 |

#### Key Findings

**1. C vs Rust Safe Semaphore: C is 1.21x faster**
- C Semaphore: 134.547 ms
- Rust Safe Semaphore: 163.188 ms
- Reasons:
  - Boundary buffer copy overhead
  - Mutex lock/unlock cost
  - Implementation strategy differences

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafe is 1.13x faster**
- Safe Semaphore: 163.188 ms
- Unsafe Semaphore: 144.552 ms
- Reasons:
  - Elimination of bounds checking
  - No boundary buffer needed (direct memory access)

**3. Barrier: Safe vs Unsafe - Unsafe is 1.13x faster**
- Rust Safe Barrier: 174.559 ms
- Rust Unsafe Barrier: 153.852 ms
- Simple synchronization pattern with lower overhead

**4. Rayon: Balanced performance - Unsafe is 0.99x faster**
- Rust Safe Rayon: 173.641 ms
- Rust Unsafe Rayon: 175.847 ms
- Practical performance despite high-level abstraction

**5. OpenMP vs Rayon: OpenMP is 1.50x faster**
- C OpenMP: 115.682 ms
- Rust Safe Rayon: 173.641 ms

### Performance Ratio Analysis

#### Safety Cost
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| Implementation | Safety Cost | Notes |
|------|-------------|------|
| Semaphore | 1.13x | Boundary buffer + Mutex (13% overhead) |
| Barrier | 1.13x | Boundary buffer + Mutex (13% overhead) |
| Rayon | 0.99x | Bounds checking (-1% overhead) |

→ **Safety cost is -1-13%**

#### Cross-Language Comparison
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| Implementation | Overhead | Analysis |
|------|----------|------|
| Semaphore | 1.21x | Implementation strategy difference is dominant |
| Barrier | 1.05x | Equivalent implementation strategy |
| Single Thread | 1.14x | Baseline comparison |

#### Parallel Speedup (vs Single Thread)

| Implementation | C Speedup | Rust Safe Speedup | Rust Unsafe Speedup |
|------|-----------|-------------------|---------------------|
| Semaphore | 2.01x | 1.90x | 2.14x |
| Barrier | 1.63x | 1.77x | 2.01x |
| OpenMP/Rayon | 2.34x | 1.78x | 1.76x |

---
