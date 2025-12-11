# Jacobi法 2D熱方程式ベンチマーク

このリポジトリは、Jacobi法による2D熱方程式シミュレーションを、RustとCで実装し、様々な並列化手法のパフォーマンスを比較するプロジェクトです。

## 実装一覧

### 共通実装（RustとC）

| 実装名 | 説明 |
|--------|------|
| **Single Thread** | シングルスレッド基準実装 |
| **Unsafe Semaphore** | アトミックカウンタによる同期（unsafe/低レベル） |
| **Safe Semaphore** | セマフォ最適化版（安全版） |
| **Barrier** | バリア同期による並列化 |
| **OpenMP / Rayon** | 高レベル並列ライブラリ（C=OpenMP、Rust=Rayon） |
| **Unsafe Optimized** | 最大限最適化されたunsafe実装 |

## システム要件

- **Rust**: 1.70以上
- **C**: GCC with pthread, OpenMP対応
- **macOS**: `brew install libomp` (OpenMP用)
- **Python**: 3.6以上（結果分析用）

## テスト実行方法

### 結果の正確性を検証

RustとC実装の計算結果が一致するかを確認するテストスクリプト：

```bash
./test_correctness.sh
```

このテストは以下を実行します：
1. 64×64グリッドで100ステップの計算を実行
2. 全6種類の実装（Single、Unsafe/Safe Semaphore、Barrier、OpenMP/Rayon、Unsafe Optimized）の結果を比較
3. RustとCの出力が完全一致するかをバイナリレベルで検証

**出力例:**
```
✓ C (single) vs Rust (single): 完全一致
✓ C (unsafe_atomic_counter) vs Rust (unsafe_atomic_counter): 完全一致
✓ C (atomic_counter) vs Rust (atomic_counter): 完全一致
✓ C (barrier) vs Rust (barrier): 完全一致
✓ C (openmp) vs Rust (rayon): 完全一致
✓ C (unsafe_parallel) vs Rust (unsafe_parallel): 完全一致
```

## ベンチマーク実行方法

### 簡単な方法：統合ベンチマークスクリプト

RustとCの両方を公平に比較するには、統合スクリプトを使用します：

```bash
# デフォルト設定で実行 (1000×1000グリッド, 100ステップ)
./run_benchmark.sh

# カスタムパラメータで実行
./run_benchmark.sh -n 500 -s 50           # 500×500グリッド, 50ステップ
./run_benchmark.sh --grid-size 2000 --steps 200  # 2000×2000グリッド, 200ステップ

# 測定回数を変更
./run_benchmark.sh -n 1000 -s 100 -i 20   # 20回測定

# すべてのオプションを表示
./run_benchmark.sh --help
```

**利用可能なオプション：**
- `-n, --grid-size N`: グリッドサイズ (N×N) [デフォルト: 1000]
- `-s, --steps NUM`: 計算ステップ数 [デフォルト: 100]
- `-i, --iterations NUM`: ベンチマーク測定回数 [デフォルト: 15]
- `-w, --warmup NUM`: ウォームアップ回数 [デフォルト: 3]
- `-c, --cooldown SEC`: クールダウン時間(秒) [デフォルト: 5]

このスクリプトは以下を自動実行します：
1. システム情報の収集
2. パラメータファイルの更新（grid.rs, jacobi_common.h）
3. RustとCのビルド
4. クールダウン期間を設けた公平なベンチマーク実行
5. 結果の自動比較と表示
6. 結果ファイルの保存（`benchmark_results/benchmark_NxN_Ssteps_TIMESTAMP.txt`）

### 個別実行

#### Rust版のみ

```bash
cd rust
cargo build --release

# デフォルト（2スレッド）で実行
cargo run --release

# スレッド数を指定して実行
cargo run --release -- 4      # 4スレッド
cargo run --release -- 8      # 8スレッド
```

#### C版のみ

```bash
cd c
make

# デフォルト（2スレッド）で実行
./jacobi_bench

# スレッド数を指定して実行
./jacobi_bench 4      # 4スレッド
./jacobi_bench 8      # 8スレッド
```

**注意**: スレッド数の指定はOpenMP（C）とRayon（Rust）にのみ影響します。他の実装（Barrier、Semaphoreなど）は常に2スレッドで動作します。

## テスト実行

### Rust版テスト

```bash
cd rust
cargo test
```

全9テストが実行されます：
- 各並列実装とシングルスレッド版の結果比較
- 決定論的動作の検証
- 境界条件の検証
- 熱源の保持確認

### C版テスト

```bash
cd c
make test
```

Rust版と同等の9テストが実行されます。

## 結果分析

ベンチマーク実行後、詳細分析スクリプトで結果を確認できます：

```bash
python3 analyze_results.py benchmark_results/benchmark_YYYYMMDD_HHMMSS.txt
```

これにより以下が生成されます：
- 詳細な比較表（コンソール出力）
- CSV形式のデータ（同じディレクトリに`.csv`ファイル）

## パラメータ設定

ベンチマークパラメータは以下のファイルで設定されています：

**Rust** (`rust/src/grid.rs`):
```rust
pub const N: usize = 1000;        // グリッドサイズ（行）
pub const M: usize = 1000;        // グリッドサイズ（列）
pub const TIME_STEPS: usize = 100; // 計算ステップ数
pub const WARMUP_STEPS: usize = 10; // ウォームアップ数
```

**C** (`c/common/jacobi_common.h`):
```c
#define N 1000                    // グリッドサイズ（行）
#define M 1000                    // グリッドサイズ（列）
#define TIME_STEPS 100            // 計算ステップ数
#define WARMUP_STEPS 10           // ウォームアップ数
```

両方を同じ値に保つことで、公平な比較が可能です。

## プロジェクト構造

```
.
├── run_benchmark.sh          # 統合ベンチマークスクリプト
├── analyze_results.py        # 結果分析スクリプト
├── benchmark_results/        # ベンチマーク結果保存先
├── CLAUDE.md                 # Claude Code向けガイド
├── README.md                 # このファイル
│
├── rust/                     # Rust実装
│   ├── src/
│   │   ├── main.rs          # ベンチマークハーネス
│   │   ├── grid.rs          # グリッド定義
│   │   └── implementations/ # 各実装
│   └── tests/
│       └── test.rs          # テストスイート
│
├── c/                        # C実装
│   ├── main.c               # ベンチマークハーネス
│   ├── test.c               # テストスイート
│   ├── Makefile             # ビルド設定
│   ├── common/              # 共通コード
│   ├── atomic_counter/      # アトミックカウンタ実装
│   ├── barrier/             # バリア実装
│   ├── omp/                 # OpenMP実装
│   ├── naive/               # ナイーブ実装
│   └── unsafe_optimized/    # Unsafe最適化実装
│
└── logic/                    # 実装ロジックのドキュメント
    └── barrier_parallel_logic.md
```

## アルゴリズム概要

### 5点ステンシル計算

2D熱伝導方程式を以下のステンシルで近似：

```
      上 (i-1, j)
       │
左 ─── ● ─── 右
(i,j-1)│(i,j+1)
       │
      下 (i+1, j)
```

Laplacian = 上 + 下 + 左 + 右 - 4×中心

### 並列化戦略

グリッドを水平に2分割し、各スレッドが上半分/下半分を担当：
- 境界行のデータを共有バッファで交換
- バリア/セマフォ/アトミック変数で同期
- ダブルバッファリングで計算と更新を分離

詳細は `logic/barrier_parallel_logic.md` を参照。

## ベンチマーク設定

- **測定回数**: 15回
- **ウォームアップ**: 3回
- **スレッド数**: 2（全実装共通）
- **キャッシュクリア**: 各試行前に5MBのダミーバッファを使用
- **クールダウン**: 実装間で5秒の待機時間

## ライセンス

このプロジェクトは実験/教育目的です。

## 貢献

バグ報告や改善提案は issue でお願いします。
