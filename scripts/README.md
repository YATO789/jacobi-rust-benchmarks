# Benchmark Scripts

このディレクトリには、Jacobi法ベンチマークを実行・検証するためのシェルスクリプトが含まれています。

## スクリプト一覧

### 1. test_correctness.sh
C言語版とRust版の実装結果の一致性を検証するテストスクリプト

**機能:**
- 小さいグリッドサイズ（64x64）で高速テスト
- 全実装の結果をバイナリ形式で比較
- Python統合による詳細な差分レポート
- カラー出力による視覚的なテスト結果表示

**使用方法:**
```bash
./test_correctness.sh
```

**実行内容:**
1. テスト用パラメータ設定（64x64グリッド、100ステップ）
2. C版とRust版のビルド
3. 各実装の実行とバイナリ出力保存
4. Python を使った詳細な結果比較
5. 許容誤差（1e-10）を超える差分の検出

**出力:**
- `test_results/c_*.bin` - C版の出力
- `test_results/rust_*.bin` - Rust版の出力
- コンソールに一致性レポート

**比較される実装:**
- Single Thread
- Unsafe Semaphore
- Safe Semaphore
- Barrier
- OpenMP/Rayon
- Unsafe Parallel

---

### 2. run_benchmark.sh
公平なベンチマーク実行スクリプト（RustとCを交互実行）

**機能:**
- システム負荷の影響を最小化するインターリーブ実行
- カスタマイズ可能なベンチマーク設定
- クールダウン時間による測定間隔の確保
- 詳細なシステム情報の記録
- Python統合による結果解析

**使用方法:**
```bash
# 基本的な使用
./run_benchmark.sh

# カスタムパラメータ
./run_benchmark.sh -n 2000 -s 200 -i 20 -w 5 -c 10

# ヘルプ表示
./run_benchmark.sh -h
```

**オプション:**
- `-n, --grid-size N` - グリッドサイズ (N×N) [デフォルト: 1000]
- `-s, --steps NUM` - 計算ステップ数 [デフォルト: 100]
- `-i, --iterations NUM` - ベンチマーク測定回数 [デフォルト: 15]
- `-w, --warmup NUM` - ウォームアップ回数 [デフォルト: 3]
- `-c, --cooldown SEC` - クールダウン時間(秒) [デフォルト: 5]
- `-h, --help` - ヘルプ表示

**実行フロー:**
1. システム情報収集（CPU、メモリ、OS）
2. パラメータファイル更新（grid.rs、jacobi_common.h）
3. C版とRust版のビルド
4. システム安定化待機
5. C版ベンチマーク実行
6. クールダウン
7. Rust版ベンチマーク実行
8. Python による結果解析と比較表生成

**出力:**
- `benchmark_results/benchmark_<size>x<size>_<steps>steps_<timestamp>.txt`
- コンソールに比較テーブル表示

---

### 3. run_full_benchmark.sh
包括的なベンチマーク測定スクリプト（複数パラメータ自動測定）

**機能:**
- 3つの測定モード（quick/standard/full）
- グリッドサイズとステップ数の組み合わせ測定
- CSV形式のサマリー出力
- 測定間の自動クールダウン

**使用方法:**
```bash
# クイックモード（約30分）
./run_full_benchmark.sh quick

# 標準モード（約3時間）
./run_full_benchmark.sh standard

# フルモード（約8-10時間）
./run_full_benchmark.sh full

# デフォルトは標準モード
./run_full_benchmark.sh
```

**測定モード:**

#### Quick Mode
- 測定時間: 約30分
- 測定項目:
  - 1024x1024, 100ステップ
  - 2048x2048, 100ステップ

#### Standard Mode
- 測定時間: 約3時間
- Phase 1（グリッドサイズの影響）:
  - 512x512, 100ステップ
  - 1024x1024, 100ステップ
  - 2048x2048, 100ステップ
- Phase 2（ステップ数の影響）:
  - 1024x1024, 50ステップ
  - 1024x1024, 100ステップ
  - 1024x1024, 200ステップ

#### Full Mode
- 測定時間: 約8-10時間
- Phase 1（グリッドサイズ sweep）:
  - 256, 512, 1024, 2048, 4096（各100ステップ）
- Phase 2（ステップ数 sweep）:
  - 1024x1024で10, 50, 100, 200, 500ステップ
- Phase 3（マトリックス測定）:
  - 512/1024/2048 × 50/100/200 の組み合わせ

**出力:**
- `benchmark_results/summary_<mode>_<timestamp>.csv`
- 各測定の詳細ファイル

**CSVフォーマット:**
```
Grid_Size,Steps,Implementation,Min_ms,Median_ms,Mean_ms,Max_ms
```

---

### 4. benchmark_grid_sizes.sh
グリッドサイズ別比較ベンチマーク（旧形式）

**機能:**
- 複数のグリッドサイズで自動測定
- 結果をCSV形式で保存
- 測定後に設定を元に戻す

**使用方法:**
```bash
./benchmark_grid_sizes.sh
```

**測定するグリッドサイズ:**
- 512x512
- 1024x1024
- 2048x2048
- 4096x4096

**各サイズで100ステップを測定**

**出力:**
- `benchmark_results/grid_size_comparison/rust_results.txt`
- `benchmark_results/grid_size_comparison/c_results.txt`

**注意:**
- このスクリプトは設定ファイルを直接書き換えます
- 測定後、2048x2048・1000ステップに設定を戻します
- より柔軟な測定には `run_full_benchmark.sh` を推奨

---

## 推奨される使用フロー

### 1. 実装の正しさを検証
```bash
./test_correctness.sh
```

### 2. 基本性能を確認
```bash
./run_benchmark.sh
```

### 3. 包括的なベンチマーク実施
```bash
# まずクイックモードで確認
./run_full_benchmark.sh quick

# 問題なければ標準モードで測定
./run_full_benchmark.sh standard

# 詳細分析が必要ならフルモード
./run_full_benchmark.sh full
```

### 4. 結果の可視化
```bash
cd ../python
python3 visualize_results.py ../benchmark_results/summary_standard_*.csv
```

---

## 依存関係

### 必須
- Bash 4.0+
- Python 3.6+
- Rust（cargo, rustc）
- C コンパイラ（gcc または clang）
- Make

### macOS固有
- OpenMPサポートには `libomp` が必要:
  ```bash
  brew install libomp
  ```

### Python依存パッケージ
結果解析スクリプト内で使用:
- 標準ライブラリのみ（追加インストール不要）

---

## トラブルシューティング

### ビルドエラー
```bash
# C版のクリーンビルド
cd ../c && make clean && make

# Rust版のクリーンビルド
cd ../rust && cargo clean && cargo build --release
```

### パラメータが反映されない
スクリプトは以下のファイルを自動更新します:
- `rust/src/grid.rs`
- `c/common/jacobi_common.h`

手動で編集した場合、スクリプトが上書きする可能性があります。

### 測定結果が不安定
- クールダウン時間を増やす: `./run_benchmark.sh -c 15`
- 測定回数を増やす: `./run_benchmark.sh -i 30`
- バックグラウンドプロセスを確認・終了

---

## スクリプト開発者向け情報

### パラメータ更新の仕組み
スクリプトは `sed` を使用して設定ファイルを書き換えます:

```bash
# Rust版
sed -i.bak "s/pub const N: usize = [0-9]*;/pub const N: usize = ${GRID_SIZE};/" rust/src/grid.rs

# C版
sed -i.bak "s/#define N [0-9]*/#define N ${GRID_SIZE}/" c/common/jacobi_common.h
```

バックアップファイル（.bak）が自動生成されます。

### 結果解析パターン
スクリプト内のPython埋め込みコードは以下のパターンで結果を抽出:
```python
pattern = r'([^:\n]+):\s+.*?中央値:\s+([0-9.]+)\s*(s|ms)'
```

出力フォーマットを変更する場合は、このパターンも更新が必要です。

---

## ファイル構成

```
scripts/
├── README.md                    # このファイル
├── test_correctness.sh          # 正確性検証スクリプト
├── run_benchmark.sh             # 単一ベンチマーク実行
├── run_full_benchmark.sh        # 包括的ベンチマーク
└── benchmark_grid_sizes.sh      # グリッドサイズ別測定（旧形式）
```
