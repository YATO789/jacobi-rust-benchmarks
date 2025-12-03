## 7. 結果

### ベンチマーク結果（64x64グリッド、1000ステップ）

#### 中央値による比較（単位: ms）

| 実装 | C言語 | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 4.757 | 6.136 | - |
| **Semaphore** | 3.934 | 4.899 | 4.834 |
| **Barrier** | 9.409 | 12.769 | 10.456 |
| **OpenMP/Rayon** | 22.921 | 13.505 | 12.881 |

#### 主要な発見

**1. C vs Rust Safe Semaphore: Cが1.25倍速い**
- C Semaphore: 3.934 ms
- Rust Safe Semaphore: 4.899 ms
- 原因:
  - 境界バッファのコピーオーバーヘッド
  - Mutexロック/アンロックのコスト
  - 実装戦略の違い

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafeが1.01倍速い**
- Safe Semaphore: 4.899 ms
- Unsafe Semaphore: 4.834 ms
- 原因:
  - 境界チェックの除去
  - 境界バッファ不要（直接メモリアクセス）

**3. Barrier: Safe vs Unsafe - Unsafeが1.22倍速い**
- Rust Safe Barrier: 12.769 ms
- Rust Unsafe Barrier: 10.456 ms
- 単純な同期パターンでオーバーヘッドが小さい

**4. Rayon: バランスの良い性能 - Unsafeが1.05倍速い**
- Rust Safe Rayon: 13.505 ms
- Rust Unsafe Rayon: 12.881 ms
- 高レベル抽象化にもかかわらず実用的な性能

**5. OpenMP vs Rayon: Rayonが1.70倍速い**
- C OpenMP: 22.921 ms
- Rust Safe Rayon: 13.505 ms

### 性能比の分析

#### 安全性のコスト
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| 実装 | Safety Cost | 備考 |
|------|-------------|------|
| Semaphore | 1.01× | 境界バッファ+Mutex (1%オーバーヘッド) |
| Barrier | 1.22× | 境界バッファ+Mutex (22%オーバーヘッド) |
| Rayon | 1.05× | 境界チェック (5%オーバーヘッド) |

→ **安全性のコストは1-22%**

#### 言語間の比較
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| 実装 | Overhead | 分析 |
|------|----------|------|
| Semaphore | 1.25× | 実装戦略の差が支配的 |
| Barrier | 1.36× | 同等の実装戦略 |
| Single Thread | 1.29× | ベースライン比較 |

#### 並列化による高速化率（シングルスレッド比）

| 実装 | C高速化率 | Rust Safe高速化率 | Rust Unsafe高速化率 |
|------|-----------|-------------------|---------------------|
| Semaphore | 1.21× | 1.25× | 1.27× |
| Barrier | 0.51× | 0.48× | 0.59× |
| OpenMP/Rayon | 0.21× | 0.45× | 0.48× |

---

### ベンチマーク結果（128x128グリッド、1000ステップ）

#### 中央値による比較（単位: ms）

| 実装 | C言語 | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 13.476 | 14.522 | - |
| **Semaphore** | 8.117 | 10.450 | 8.694 |
| **Barrier** | 15.821 | 18.916 | 15.158 |
| **OpenMP/Rayon** | 23.707 | 19.223 | 18.469 |

#### 主要な発見

**1. C vs Rust Safe Semaphore: Cが1.29倍速い**
- C Semaphore: 8.117 ms
- Rust Safe Semaphore: 10.450 ms
- 原因:
  - 境界バッファのコピーオーバーヘッド
  - Mutexロック/アンロックのコスト
  - 実装戦略の違い

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafeが1.20倍速い**
- Safe Semaphore: 10.450 ms
- Unsafe Semaphore: 8.694 ms
- 原因:
  - 境界チェックの除去
  - 境界バッファ不要（直接メモリアクセス）

**3. Barrier: Safe vs Unsafe - Unsafeが1.25倍速い**
- Rust Safe Barrier: 18.916 ms
- Rust Unsafe Barrier: 15.158 ms
- 単純な同期パターンでオーバーヘッドが小さい

**4. Rayon: バランスの良い性能 - Unsafeが1.04倍速い**
- Rust Safe Rayon: 19.223 ms
- Rust Unsafe Rayon: 18.469 ms
- 高レベル抽象化にもかかわらず実用的な性能

**5. OpenMP vs Rayon: Rayonが1.23倍速い**
- C OpenMP: 23.707 ms
- Rust Safe Rayon: 19.223 ms

### 性能比の分析

#### 安全性のコスト
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| 実装 | Safety Cost | 備考 |
|------|-------------|------|
| Semaphore | 1.20× | 境界バッファ+Mutex (20%オーバーヘッド) |
| Barrier | 1.25× | 境界バッファ+Mutex (25%オーバーヘッド) |
| Rayon | 1.04× | 境界チェック (4%オーバーヘッド) |

→ **安全性のコストは4-25%**

#### 言語間の比較
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| 実装 | Overhead | 分析 |
|------|----------|------|
| Semaphore | 1.29× | 実装戦略の差が支配的 |
| Barrier | 1.20× | 同等の実装戦略 |
| Single Thread | 1.08× | ベースライン比較 |

#### 並列化による高速化率（シングルスレッド比）

| 実装 | C高速化率 | Rust Safe高速化率 | Rust Unsafe高速化率 |
|------|-----------|-------------------|---------------------|
| Semaphore | 1.66× | 1.39× | 1.67× |
| Barrier | 0.85× | 0.77× | 0.96× |
| OpenMP/Rayon | 0.57× | 0.76× | 0.79× |

---

### ベンチマーク結果（512x512グリッド、1000ステップ）

#### 中央値による比較（単位: ms）

| 実装 | C言語 | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 76.215 | 77.546 | - |
| **Semaphore** | 37.877 | 43.697 | 40.656 |
| **Barrier** | 46.166 | 49.660 | 42.600 |
| **OpenMP/Rayon** | 39.097 | 48.003 | 49.633 |

#### 主要な発見

**1. C vs Rust Safe Semaphore: Cが1.15倍速い**
- C Semaphore: 37.877 ms
- Rust Safe Semaphore: 43.697 ms
- 原因:
  - 境界バッファのコピーオーバーヘッド
  - Mutexロック/アンロックのコスト
  - 実装戦略の違い

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafeが1.07倍速い**
- Safe Semaphore: 43.697 ms
- Unsafe Semaphore: 40.656 ms
- 原因:
  - 境界チェックの除去
  - 境界バッファ不要（直接メモリアクセス）

**3. Barrier: Safe vs Unsafe - Unsafeが1.17倍速い**
- Rust Safe Barrier: 49.660 ms
- Rust Unsafe Barrier: 42.600 ms
- 単純な同期パターンでオーバーヘッドが小さい

**4. Rayon: Balanced performance - Safeが1.03倍速い**
- Rust Safe Rayon: 48.003 ms
- Rust Unsafe Rayon: 49.633 ms
- 高レベル抽象化にもかかわらず実用的な性能

**5. OpenMP vs Rayon: OpenMPが1.23倍速い**
- C OpenMP: 39.097 ms
- Rust Safe Rayon: 48.003 ms

### 性能比の分析

#### 安全性のコスト
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| 実装 | Safety Cost | 備考 |
|------|-------------|------|
| Semaphore | 1.07× | 境界バッファ+Mutex (7%オーバーヘッド) |
| Barrier | 1.17× | 境界バッファ+Mutex (17%オーバーヘッド) |
| Rayon | 0.97× | 境界チェック (-3%オーバーヘッド) |

→ **安全性のコストは-3-17%**

#### 言語間の比較
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| 実装 | Overhead | 分析 |
|------|----------|------|
| Semaphore | 1.15× | 実装戦略の差が支配的 |
| Barrier | 1.08× | 同等の実装戦略 |
| Single Thread | 1.02× | ベースライン比較 |

#### 並列化による高速化率（シングルスレッド比）

| 実装 | C高速化率 | Rust Safe高速化率 | Rust Unsafe高速化率 |
|------|-----------|-------------------|---------------------|
| Semaphore | 2.01× | 1.77× | 1.91× |
| Barrier | 1.65× | 1.56× | 1.82× |
| OpenMP/Rayon | 1.95× | 1.62× | 1.56× |

---

### ベンチマーク結果（1024x1024グリッド、1000ステップ）

#### 中央値による比較（単位: ms）

| 実装 | C言語 | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **Single Thread** | 270.432 | 309.318 | - |
| **Semaphore** | 134.547 | 163.188 | 144.552 |
| **Barrier** | 165.879 | 174.559 | 153.852 |
| **OpenMP/Rayon** | 115.682 | 173.641 | 175.847 |

#### 主要な発見

**1. C vs Rust Safe Semaphore: Cが1.21倍速い**
- C Semaphore: 134.547 ms
- Rust Safe Semaphore: 163.188 ms
- 原因:
  - 境界バッファのコピーオーバーヘッド
  - Mutexロック/アンロックのコスト
  - 実装戦略の違い

**2. Rust Safe vs Rust Unsafe Semaphore: Unsafeが1.13倍速い**
- Safe Semaphore: 163.188 ms
- Unsafe Semaphore: 144.552 ms
- 原因:
  - 境界チェックの除去
  - 境界バッファ不要（直接メモリアクセス）

**3. Barrier: Safe vs Unsafe - Unsafeが1.13倍速い**
- Rust Safe Barrier: 174.559 ms
- Rust Unsafe Barrier: 153.852 ms
- 単純な同期パターンでオーバーヘッドが小さい

**4. Rayon: Balanced performance - Safeが1.01倍速い**
- Rust Safe Rayon: 173.641 ms
- Rust Unsafe Rayon: 175.847 ms
- 高レベル抽象化にもかかわらず実用的な性能

**5. OpenMP vs Rayon: OpenMPが1.50倍速い**
- C OpenMP: 115.682 ms
- Rust Safe Rayon: 173.641 ms

### 性能比の分析

#### 安全性のコスト
```
Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)
```

| 実装 | Safety Cost | 備考 |
|------|-------------|------|
| Semaphore | 1.13× | 境界バッファ+Mutex (13%オーバーヘッド) |
| Barrier | 1.13× | 境界バッファ+Mutex (13%オーバーヘッド) |
| Rayon | 0.99× | 境界チェック (-1%オーバーヘッド) |

→ **安全性のコストは-1-13%**

#### 言語間の比較
```
Language Overhead = Time(Rust Safe) / Time(C)
```

| 実装 | Overhead | 分析 |
|------|----------|------|
| Semaphore | 1.21× | 実装戦略の差が支配的 |
| Barrier | 1.05× | 同等の実装戦略 |
| Single Thread | 1.14× | ベースライン比較 |

#### 並列化による高速化率（シングルスレッド比）

| 実装 | C高速化率 | Rust Safe高速化率 | Rust Unsafe高速化率 |
|------|-----------|-------------------|---------------------|
| Semaphore | 2.01× | 1.90× | 2.14× |
| Barrier | 1.63× | 1.77× | 2.01× |
| OpenMP/Rayon | 2.34× | 1.78× | 1.76× |

---
