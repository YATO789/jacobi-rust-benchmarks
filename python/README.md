# Benchmark Analysis Tools

このディレクトリには、Jacobi法ベンチマーク結果を解析・可視化するためのPythonスクリプトが含まれています。

## スクリプト一覧

### 1. visualize_results.py
包括的なベンチマーク結果の可視化ツール

**機能:**
- グリッドサイズ vs 実行時間のグラフ
- ステップ数 vs 実行時間のグラフ
- 相対性能比較（Speedup）
- 安全性コストのヒートマップ
- サマリーテーブルの生成

**使用方法:**
```bash
python3 visualize_results.py <summary.csv>
```

**依存関係:**
- pandas
- matplotlib
- numpy

**出力:**
- `visualizations/grid_size_comparison.png` - グリッドサイズ比較
- `visualizations/time_steps_comparison.png` - ステップ数比較
- `visualizations/speedup_comparison.png` - Speedup比較
- `visualizations/safety_cost_heatmap.png` - 安全性コストヒートマップ
- `visualizations/summary_table.txt` - サマリーテーブル

---

### 2. plot_benchmark.py
カテゴリー別のベンチマーク結果グラフ生成

**機能:**
- 実装カテゴリー別（Sequential, Barrier, Semaphore, OpenMP/Rayon）のグラフ
- 全実装をまとめた総合グラフ
- ログスケール表示

**使用方法:**
```bash
python3 plot_benchmark.py
```

**依存関係:**
- matplotlib

**ハードコードされた設定:**
- ベンチマークファイルパス（スクリプト内で指定）
- グリッドサイズ: [64, 128, 512, 1024]

**出力:**
- `benchmark_results/performance_comparison_categorized.png` - カテゴリー別グラフ
- `benchmark_results/performance_comparison_all.png` - 全体グラフ

---

### 3. create_table.py
Markdown形式のベンチマーク結果テーブル生成

**機能:**
- 実装別の中央値時間をテーブル化
- Single Threadに対する高速化率の計算
- Markdown形式で出力

**使用方法:**
```bash
python3 create_table.py
```

**依存関係:**
- なし（標準ライブラリのみ）

**ハードコードされた設定:**
- ベンチマークファイルパス（スクリプト内で指定）
- グリッドサイズ: [64, 128, 512, 1024]

**出力:**
- `benchmark_results/benchmark_table.md` - Markdownテーブル
- コンソールにも結果を表示

---

### 4. analyze_results.py
詳細なパフォーマンス分析レポート生成

**機能:**
- C言語 vs Rust Safe Semaphoreの比較
- Rust Safe vs Rust Unsafeの比較
- 安全性コストの計算
- 言語間オーバーヘッドの分析
- 並列化による高速化率（Speedup）の計算

**使用方法:**
```bash
python3 analyze_results.py
```

**依存関係:**
- なし（標準ライブラリのみ）

**ハードコードされた設定:**
- ベンチマークファイルパス（スクリプト内で指定）
- グリッドサイズ: [64, 128, 512, 1024]

**出力:**
- `benchmark_results/analysis_results.md` - 詳細分析レポート

**分析内容:**
- グリッドサイズごとの実装比較テーブル
- 主要な発見（Key Findings）
- 安全性コスト（Safety Cost）
- 言語間比較（Cross-Language Comparison）
- 並列化による高速化率（Parallel Speedup）

---

## 依存関係のインストール

必要なPythonパッケージをインストールするには:

```bash
pip3 install pandas matplotlib numpy
```

または、最小限の依存関係のみの場合:

```bash
# visualize_results.py 用
pip3 install pandas matplotlib numpy

# plot_benchmark.py 用
pip3 install matplotlib

# create_table.py, analyze_results.py は標準ライブラリのみを使用
```

---

## 使用の流れ

### 1. ベンチマーク実行後の基本的な可視化

```bash
# テーブル生成
python3 create_table.py

# グラフ生成
python3 plot_benchmark.py

# 詳細分析
python3 analyze_results.py
```

### 2. CSV形式のサマリーデータがある場合

```bash
python3 visualize_results.py path/to/summary.csv
```

---

## 注意事項

- `plot_benchmark.py`、`create_table.py`、`analyze_results.py` は、特定のベンチマークファイルパスがハードコードされています
- ファイル名やパスを変更した場合は、スクリプト内の設定を更新してください
- `visualize_results.py` はCSV形式の入力を期待し、より柔軟に使用できます

---

## ファイル構成

```
python/
├── README.md                    # このファイル
├── visualize_results.py         # 包括的な可視化ツール（CSV入力）
├── plot_benchmark.py            # カテゴリー別グラフ生成
├── create_table.py              # Markdownテーブル生成
└── analyze_results.py           # 詳細分析レポート生成
```
