# Jacobi法実装の詳細比較

## 目的
C言語とRust（SafeとUnsafe）の実装が、同じアルゴリズムを使用していることを検証し、公平なベンチマーク比較を行う。

---

## 共通仕様

### グリッド設定
- **サイズ**: N × M (デフォルト: 1000 × 1000)
- **境界条件**: 全境界で0.0固定（Dirichlet境界条件）
- **熱源**: グリッド中央 (N/2, M/2) で100.0固定
- **初期状態**: 熱源以外は全て0.0

### 計算パラメータ
```
ALPHA = 1.0         // 熱拡散係数
DT = 0.01           // 時間刻み
DX = 1.0            // 空間刻み
factor = ALPHA * DT / (DX * DX) = 0.01
```

### 5点ステンシル（Laplacian計算）
```
laplacian = u[i+1,j] + u[i-1,j] + u[i,j+1] + u[i,j-1] - 4*u[i,j]
u_new[i,j] = u[i,j] + factor * laplacian
```

### ダブルバッファリング
- 各ステップで読み取り元(src)と書き込み先(dst)を交互に入れ替え
- 奇数ステップ後は結果をgrid_aにコピーバック

---

## 実装1: セマフォ方式（Atomic Counter Synchronization）

### アルゴリズム概要
- グリッドを水平に2分割（上半分: row 1～mid, 下半分: row mid～N-1）
- 各スレッドが独立した領域を計算
- アトミックカウンターで同期（境界データの読み取り完了を待つ）

### C言語実装 (`c/atomic_counter/jacobi_atomic_counter.c`)

#### 主要な特徴
```c
// 1. スレッド分割
上半分: row_start=1, row_end=mid
下半分: row_start=mid, row_end=N-1

// 2. 同期メカニズム
atomic_size_t count_u, count_l;  // スピン待機用カウンター
atomic_store_explicit(my_counter, step, memory_order_release);
while (atomic_load_explicit(other_counter, memory_order_acquire) < step) {
    CPU_RELAX();  // x86ではPAUSE命令
}

// 3. 計算ループ（境界バッファなし、直接読み取り）
for (int i = r_start; i < r_end; i++) {
    double *src_curr = src + i * M;
    double *src_up   = src + (i - 1) * M;
    double *src_down = src + (i + 1) * M;
    double *dst_curr = dst + i * M;

    for (int j = 1; j < M - 1; j++) {
        double v = src_curr[j];
        double laplacian = src_curr[j+1] + src_curr[j-1]
                         + src_down[j] + src_up[j] - 4.0*v;
        dst_curr[j] = v + factor * laplacian;
    }
}
```

#### 同期タイミング
- ステップ終了時に1回のみ同期（書き込み完了通知）
- 境界バッファへのコピー不要（src から直接読み取り）

---

### Rust Safe実装 (`rust/src/implementations/safe/atomic_counter/atomic_counter.rs`)

#### 主要な特徴
```rust
// 1. スレッド分割（同じ）
上半分: row 1～mid
下半分: row mid～N-1

// 2. 同期メカニズム
AtomicUsize (upper_ready, lower_ready, upper_done, lower_done)
// 4つのアトミック変数を使用
my_counter.store(step, Ordering::Release);
while other_counter.load(Ordering::Acquire) < step {
    std::hint::spin_loop();  // スピンループ最適化
}

// 3. 境界バッファ（Mutex保護）
Arc<Mutex<Vec<f64>>> boundary_mid_minus_1;  // 上→下
Arc<Mutex<Vec<f64>>> boundary_mid;          // 下→上

// 境界データの書き込み
{
    let mut writer = my_bound_writer.lock().unwrap();
    writer.copy_from_slice(&src[row_idx*M..(row_idx+1)*M]);
}

// 境界行の計算（バッファから読み取り）
{
    let reader = other_bound_reader.lock().unwrap();
    let down_val = reader[j];
    let laplacian = src[idx-M] + down_val + src[idx-1] + src[idx+1] - 4.0*src[idx];
}
```

#### 同期タイミング
- ステップ開始時: 境界データ書き込み完了を通知（Ready）
- 計算後: 境界データ読み取り完了を通知（Done）
- 合計2回の同期ポイント

#### **差異ポイント**
- C言語: 境界バッファなし、直接読み取り
- Rust Safe: Mutex保護の境界バッファ使用（安全性のため）

---

### Rust Unsafe実装 (`rust/src/implementations/unsafe/unsafe_atomic_counter.rs`)

#### 主要な特徴
```rust
// 1. GridHandleによる生ポインタ管理
#[derive(Clone, Copy)]
struct GridHandle(NonNull<Grid>);
unsafe impl Send for GridHandle {}
unsafe impl Sync for GridHandle {}

// 2. 同期メカニズム（C言語に近い）
Arc<AlignedAtomic(AtomicUsize)>  // キャッシュライン整列
wait_for_step(&ready, step);     // スピン待機
signal.store(step+1, Ordering::Release);

// 3. 計算ループ（直接メモリアクセス）
unsafe fn jacobi_band(
    src: GridHandle,
    dst: GridHandle,
    row_start: usize,
    row_end: usize,
    factor: f64,
    enforce_heat_source: bool,
) {
    let src_ref = src.0.as_ref();
    let dst_ref = dst.0.as_mut();

    // get_uncheckedで境界チェックなし
    for i in row_start..row_end {
        update_row(src_ref, dst_ref, i, 1, M-1, factor);
    }
}
```

#### **特徴**
- C言語と同等のメモリアクセスパターン
- 境界チェックなし（`get_unchecked`）
- 境界バッファなし、直接読み取り

---

## 実装2: バリア方式（Barrier Synchronization）

### アルゴリズム概要
- グリッドを水平に2分割
- 各ステップでバリア同期を使用
- 境界データの読み書きをバリアで保護

### C言語実装 (`c/barrier/jacobi_barrier.c`)

```c
// 1. バリア同期（macOSでは手動実装）
pthread_barrier_t barrier;
pthread_barrier_wait(&barrier);

// 2. 計算パターン
for (int t = 0; t < steps; t++) {
    // 計算
    for (int i = r_start; i < r_end; i++) {
        for (int j = 1; j < M-1; j++) {
            int idx = i*M + j;
            double laplacian = src[(i+1)*M+j] + src[(i-1)*M+j]
                             + src[i*M+(j+1)] + src[i*M+(j-1)] - 4.0*src[idx];
            dst[idx] = src[idx] + factor*laplacian;
        }
    }

    // 熱源設定
    if (mid_row >= start_row && mid_row < end_row) {
        dst[mid_row*M + (M/2)] = 100.0;
    }

    pthread_barrier_wait(&barrier);  // 計算完了待ち

    // ポインタスワップ（ローカル変数）
    double *temp = src; src = dst; dst = temp;
}
```

#### **特徴**
- 境界データを直接読み取り（境界バッファなし）
- バリアで全スレッドの計算完了を保証
- ローカル変数でポインタをスワップ

---

### Rust Safe実装 (`rust/src/implementations/safe/barrier/barrier_parallel.rs`)

```rust
// 1. バリア同期
let barrier = Arc::new(Barrier::new(2));
barrier.wait();

// 2. 境界バッファ（Mutex保護）
Arc<Mutex<Vec<f64>>> boundary_mid_minus_1;
Arc<Mutex<Vec<f64>>> boundary_mid;

// 3. 計算パターン
for _step in 0..steps {
    // 境界データ書き込み
    {
        let mut writer = bound_write.lock().unwrap();
        writer.copy_from_slice(&src[row_idx*M..(row_idx+1)*M]);
    }

    barrier.wait();  // 境界データ準備完了

    // 内部領域計算
    for i in 1..rows-1 {
        for j in 1..M-1 {
            let idx = i*M + j;
            let laplacian = src[idx-M] + src[idx+M] + src[idx-1] + src[idx+1] - 4.0*src[idx];
            dst[idx] = src[idx] + factor*laplacian;
        }
    }

    // 境界行計算（バッファから読み取り）
    {
        let reader = bound_read.lock().unwrap();
        let i = rows-1;
        for j in 1..M-1 {
            let idx = i*M + j;
            let down_val = reader[j];
            let laplacian = src[idx-M] + down_val + src[idx-1] + src[idx+1] - 4.0*src[idx];
            dst[idx] = src[idx] + factor*laplacian;
        }
    }

    barrier.wait();  // 計算完了待ち
    std::mem::swap(&mut src, &mut dst);
}
```

#### **差異ポイント**
- C言語: 境界データ直接読み取り
- Rust Safe: Mutex保護の境界バッファ使用

---

### Rust Unsafe実装 (`rust/src/implementations/unsafe/barrier_unsafe.rs`)

```rust
// 1. GridPtrラッパー
struct GridPtr { data: *mut f64 }
unsafe impl Send for GridPtr {}
unsafe impl Sync for GridPtr {}

// 2. バリア同期
let barrier = Arc::new(Barrier::new(2));
barrier.wait();

// 3. 計算（生ポインタアクセス）
unsafe fn jacobi_band_raw(
    src: *const f64,
    dst: *mut f64,
    row_start: usize,
    row_end: usize,
    factor: f64,
    enforce_heat_source: bool,
) {
    for i in row_start..row_end {
        unsafe {
            let src_curr = src.add(i*M);
            let src_up = src.add((i-1)*M);
            let src_down = src.add((i+1)*M);
            let dst_row = dst.add(i*M);

            for j in 1..M-1 {
                let v = *src_curr.add(j);
                let laplacian = *src_curr.add(j+1) + *src_curr.add(j-1)
                              + *src_down.add(j) + *src_up.add(j) - 4.0*v;
                *dst_row.add(j) = v + factor*laplacian;
            }
        }
    }
}
```

#### **特徴**
- C言語とほぼ同等のポインタ操作
- 境界チェックなし
- 境界バッファなし

---

## 主要な実装差異のまとめ

### 1. セマフォ方式

| 特性 | C言語 | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **境界バッファ** | なし（直接読み取り） | あり（Mutex保護） | なし（直接読み取り） |
| **同期ポイント** | 1回/ステップ | 2回/ステップ | 1回/ステップ |
| **メモリアクセス** | 生ポインタ | 境界チェック付き | `get_unchecked` |
| **同期プリミティブ** | `atomic_size_t` | `AtomicUsize` | `AtomicUsize` |

### 2. バリア方式

| 特性 | C言語 | Rust Safe | Rust Unsafe |
|------|-------|-----------|-------------|
| **境界バッファ** | なし（直接読み取り） | あり（Mutex保護） | なし（直接読み取り） |
| **同期ポイント** | 1回/ステップ | 2回/ステップ | 1回/ステップ |
| **メモリアクセス** | 生ポインタ | 境界チェック付き | 生ポインタ |
| **バリア実装** | pthread_barrier | `std::sync::Barrier` | `std::sync::Barrier` |

---

## 安全性のコスト分析

### Rust Safe版のオーバーヘッド源

1. **境界バッファのコピー**
   - 各ステップで境界行（M要素）をMutexバッファにコピー
   - Mutexロック/アンロックのコスト
   - メモリコピー操作

2. **境界チェック**
   - 配列アクセス時の境界チェック（panics回避）
   - 特に内側ループで影響大

3. **追加の同期ポイント**
   - C/Unsafe: 1回/ステップ
   - Rust Safe: 2回/ステップ（Ready + Done）

### Rust Unsafe版の最適化

1. **境界チェック除去**
   - `get_unchecked` / `get_unchecked_mut` 使用
   - ポインタ演算による直接アクセス

2. **境界バッファ除去**
   - C言語と同様に直接メモリ読み取り
   - Mutexロック不要

3. **同期の最適化**
   - 1回/ステップに削減

---

## ベンチマーク上の注意点

### 公平な比較のために確認すべき事項

1. **アルゴリズムの同等性** ✅
   - すべて5点ステンシル
   - すべてダブルバッファリング
   - グリッド分割方法が同一

2. **計算精度** ✅
   - factor = 0.01 で統一
   - 同じ数値型（double / f64）
   - 同じ境界条件

3. **スレッド数** ✅
   - すべて2スレッド固定

4. **同期方式の違い** ⚠️
   - **C vs Rust Safe**: 境界バッファの有無により実装が異なる
   - **C vs Rust Unsafe**: ほぼ同等（公平な比較可能）
   - **Rust Safe vs Rust Unsafe**: 安全性コストの定量化が可能

---

## 結論

### 実装の同等性

**高い同等性（公平な比較）:**
- C言語 ⇔ Rust Unsafe: ほぼ完全に同等のアルゴリズム
- Rust Safe (Semaphore) ⇔ Rust Unsafe (Semaphore): 同じ同期戦略、境界バッファの有無のみ
- Rust Safe (Barrier) ⇔ Rust Unsafe (Barrier): 同じ同期戦略、境界バッファの有無のみ

**実装差異あり（注意が必要）:**
- C言語 ⇔ Rust Safe: 境界バッファと追加同期により実装が異なる
  - これは**Rustの安全性保証のための設計差異**
  - ベンチマーク結果の差はこの安全性コストを反映

### ベンチマーク結果の解釈

1. **C vs Rust Unsafe**: 言語とコンパイラの純粋な性能差
2. **Rust Safe vs Rust Unsafe**: Rustの安全性機能のオーバーヘッド
3. **C vs Rust Safe**: 安全性と言語特性の組み合わせ効果

これらの比較により、安全性のコストを多角的に測定可能。
