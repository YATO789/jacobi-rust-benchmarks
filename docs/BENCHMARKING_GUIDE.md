# ベンチマーク実行ガイド

## クイックスタート

### 1. 単発測定

```bash
# 基本測定（1024×1024グリッド、100ステップ）
./run_benchmark.sh -n 1024 -s 100

# カスタムパラメータ
./run_benchmark.sh -n 2048 -s 200
```

### 2. 一括測定

```bash
# クイックモード（約30分）- 基本性能確認
./run_full_benchmark.sh quick

# 標準モード（約3時間）- 主要パラメータ測定
./run_full_benchmark.sh standard

# フルモード（約8-10時間）- 完全測定
./run_full_benchmark.sh full
```

### 3. 結果の可視化

```bash
# Pythonパッケージのインストール（初回のみ）
pip3 install pandas matplotlib numpy

# グラフ生成
python3 visualize_results.py benchmark_results/summary_standard_*.csv
```

---

## 測定モード詳細

### Quick Mode（クイックモード）

**測定パラメータ:**
- Grid: 1024×1024, Steps: 100
- Grid: 2048×2048, Steps: 100

**推定時間:** 約30分

**目的:**
- 基本性能の確認
- システム動作確認
- デバッグ用

### Standard Mode（標準モード）

**測定パラメータ:**

Phase 1: グリッドサイズの影響
- Grid: 512×512, Steps: 100
- Grid: 1024×1024, Steps: 100
- Grid: 2048×2048, Steps: 100

Phase 2: ステップ数の影響
- Grid: 1024×1024, Steps: 50
- Grid: 1024×1024, Steps: 100
- Grid: 1024×1024, Steps: 200

**推定時間:** 約3時間

**目的:**
- 主要な性能特性の把握
- 研究発表用データ収集
- **推奨測定セット**

### Full Mode（フルモード）

**測定パラメータ:**

Phase 1: グリッドサイズsweep
- 256, 512, 1024, 2048, 4096 (Steps: 100)

Phase 2: ステップ数sweep
- 10, 50, 100, 200, 500 (Grid: 1024×1024)

Phase 3: マトリックス測定
- Grid×Steps: 512×{50,100,200}, 1024×{50,100,200}, 2048×{50,100,200}

**推定時間:** 約8-10時間

**目的:**
- 完全なパラメータ空間の探索
- 論文用詳細データ
- スケーラビリティ分析

---

## 測定前の準備

### システム設定

```bash
# 1. バックグラウンドプロセスの確認
top -l 1 | head -20

# 2. CPU温度の確認（macOS）
sudo powermetrics -n 1 -i 1000 | grep -i "CPU die temperature"

# 3. 電源接続確認
system_profiler SPPowerDataType | grep "Connected"
```

### 推奨環境

- ✅ 電源接続状態
- ✅ バッテリー節約モードOFF
- ✅ 他のアプリケーションを閉じる
- ✅ システムアップデート無効化
- ✅ 同じ時間帯に測定（温度影響の最小化）

---

## 結果の確認

### 個別結果ファイル

```bash
# 最新の結果を確認
ls -lt benchmark_results/*.txt | head -5

# 特定の結果を表示
cat benchmark_results/benchmark_1024x1024_100steps_*.txt
```

### サマリーCSV

```bash
# CSVファイルの確認
cat benchmark_results/summary_standard_*.csv

# スプレッドシートで開く
open benchmark_results/summary_standard_*.csv
```

### 生成されるグラフ

1. **grid_size_comparison.png**
   - X軸: グリッドサイズ（対数スケール）
   - Y軸: 実行時間（ms、対数スケール）
   - 各実装の性能曲線

2. **time_steps_comparison.png**
   - X軸: ステップ数
   - Y軸: 実行時間（ms）
   - 同期オーバーヘッドの可視化

3. **speedup_comparison.png**
   - 横棒グラフ
   - C Single Threadを基準とした相対性能

4. **safety_cost_heatmap.png**
   - Rust SafeとUnsafeの性能差をパーセント表示
   - グリッドサイズとステップ数の組み合わせ

5. **summary_table.txt**
   - 数値テーブル（テキスト形式）

---

## トラブルシューティング

### ビルドエラー

```bash
# C版の再ビルド
cd c && make clean && make

# Rust版の再ビルド
cd rust && cargo clean && cargo build --release
```

### 測定が遅い

```bash
# システムリソースの確認
htop

# ディスクI/Oの確認
iostat 1 10
```

### 結果の検証

```bash
# テストの実行
cd c && make test
cd rust && cargo test

# 手動実行
./c/jacobi_bench
./rust/target/release/jacobi-rust
```

---

## データ分析例

### Excelでの分析

1. CSVファイルをExcelで開く
2. ピボットテーブルで集計
3. グラフ作成

### Pythonでの分析

```python
import pandas as pd

# データ読み込み
df = pd.read_csv('benchmark_results/summary_standard_*.csv')

# 統計分析
print(df.groupby('Implementation')['Median_ms'].describe())

# 相関分析
print(df[['Grid_Size', 'Steps', 'Median_ms']].corr())
```

### R での分析

```r
library(tidyverse)

# データ読み込み
df <- read_csv('benchmark_results/summary_standard_*.csv')

# 可視化
ggplot(df, aes(x=Grid_Size, y=Median_ms, color=Implementation)) +
  geom_line() +
  scale_x_log10() +
  scale_y_log10()
```

---

## 推奨測定フロー

### 研究発表用（最小限）

```bash
# 1. クイック測定で動作確認
./run_full_benchmark.sh quick

# 2. 結果確認
ls -lt benchmark_results/

# 3. 問題なければ標準測定
./run_full_benchmark.sh standard

# 4. 可視化
python3 visualize_results.py benchmark_results/summary_standard_*.csv
```

### 論文用（完全測定）

```bash
# 1. システム準備
# - 電源接続
# - 他のアプリ終了
# - 測定環境記録

# 2. フル測定（夜間実行推奨）
nohup ./run_full_benchmark.sh full > full_benchmark.log 2>&1 &

# 3. 進捗確認
tail -f full_benchmark.log

# 4. 完了確認
ls -lt benchmark_results/

# 5. 可視化と分析
python3 visualize_results.py benchmark_results/summary_full_*.csv
```

---

## 結果の解釈ガイド

### 性能比較指標

**Speedup（高速化率）:**
- Speedup > 1.0: 実装Aが実装Bより速い
- Speedup = 1.0: 同等
- Speedup < 1.0: 実装Bの方が速い

**Safety Cost（安全性コスト）:**
```
Safety Cost = (Time_Safe - Time_Unsafe) / Time_Unsafe × 100%
```
- 5%未満: 安全性コストは小さい
- 5-15%: 中程度のコスト
- 15%以上: 大きなコスト

### 期待される傾向

**グリッドサイズの影響:**
- 小サイズ: キャッシュヒット率高い → 最速
- 大サイズ: メモリ帯域律速 → 遅い

**ステップ数の影響:**
- Rust Safe: 2回同期/ステップ → 線形増加
- C/Unsafe: 1回同期/ステップ → より緩やかな増加

**実装方式:**
- Semaphore: スピン待機、低オーバーヘッド
- Barrier: 条件変数、やや重い
- Rayon: ワークスティーリング、スケーラブル

---

## よくある質問

**Q: 測定時間を短縮できますか？**

A: `-i` オプションで測定回数を減らせます:
```bash
./run_benchmark.sh -n 1024 -s 100 -i 5  # 15回→5回
```
ただし統計的信頼性は低下します。

**Q: 特定の実装だけ測定できますか？**

A: 現在のスクリプトは全実装を測定します。個別測定は手動実行してください:
```bash
./c/jacobi_bench
./rust/target/release/jacobi-rust
```

**Q: 結果が不安定です**

A:
1. システム負荷を確認
2. クールダウン時間を延長（`-c` オプション）
3. 測定回数を増やす（`-i` オプション）
4. 電源接続とバッテリー設定を確認

---

## 次のステップ

測定完了後:

1. **IMPLEMENTATION_COMPARISON.md** を参照して実装の違いを理解
2. **BENCHMARK_STRATEGY.md** でパラメータ選定の理論的根拠を確認
3. 結果を論文やプレゼンテーションに活用

良い測定を！
