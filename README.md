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
| **Rayon / OpenMP** | 高レベル並列ライブラリ（Rust=Rayon、C=OpenMP） |
| **Channel / Naive** | メッセージパッシング / ナイーブ実装 |
| **Unsafe Optimized** | 最大限最適化されたunsafe実装 |

## システム要件

- **Rust**: 1.70以上
- **C**: GCC with pthread, OpenMP対応
- **macOS**: `brew install libomp` (OpenMP用)
- **Python**: 3.6以上（結果分析用）

## ベンチマーク実行方法

### 簡単な方法：統合ベンチマークスクリプト

RustとCの両方を公平に比較するには、統合スクリプトを使用します：

```bash
./run_benchmark.sh
```

このスクリプトは以下を自動実行します：
1. システム情報の収集
2. RustとCのビルド
3. クールダウン期間を設けた公平なベンチマーク実行
4. 結果の自動比較と表示
5. 結果ファイルの保存（`benchmark_results/`）

### 個別実行

#### Rust版のみ

```bash
cd rust
cargo build --release
cargo run --release
```

#### C版のみ

```bash
cd c
make
make run
```

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
│   ├── semaphore/           # セマフォ実装
│   ├── barrier/             # バリア実装
│   ├── omp/                 # OpenMP実装
│   ├── naive/               # ナイーブ実装
│   ├── unsafe_semaphore/    # Unsafeセマフォ実装
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
