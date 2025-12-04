# Documentation

このディレクトリには、Jacobi法ベンチマークプロジェクトの技術文書が含まれています。

## ドキュメント一覧

### IMPLEMENTATION_COMPARISON.md
各並列化手法の実装比較と特徴分析

**内容:**
- Barrier方式の実装詳細
- Semaphore方式の実装詳細
- Channel方式の実装詳細
- OpenMP/Rayon方式の実装詳細
- Safe vs Unsafeの実装差異
- 各手法の利点と欠点

**対象読者:**
- 実装の詳細を理解したい開発者
- 並列化手法を選択する際の参考にしたい方

---

### BENCHMARK_STRATEGY.md
ベンチマーク戦略と測定方法論

**内容:**
- ベンチマーク設計の原則
- 公平な比較のための工夫
- 測定誤差の低減手法
- 統計的な結果の解釈方法
- パラメータ選択の根拠

**対象読者:**
- ベンチマーク結果を解釈したい方
- 測定方法の妥当性を確認したい方
- 独自のベンチマークを設計したい方

---

### BENCHMARKING_GUIDE.md
ベンチマーク実行の実践ガイド

**内容:**
- ベンチマーク実行の準備
- スクリプトの使い方
- 結果の読み方と解釈
- トラブルシューティング
- ベストプラクティス

**対象読者:**
- ベンチマークを初めて実行する方
- 正確な測定結果を得たい方

---

### PRESENTATION.md
研究発表・プレゼンテーション用資料

**内容:**
- プロジェクト概要のサマリー
- 主要な発見と結果
- グラフと図表
- 結論と今後の課題

**対象読者:**
- プレゼンテーション資料を作成する方
- プロジェクトの概要を素早く理解したい方

---

### SINGLE_THREAD_FLOW.md
シングルスレッド実装のアルゴリズムフロー

**内容:**
- Jacobi法の基本アルゴリズム
- 5点ステンシル計算の詳細
- ダブルバッファリングの実装
- メモリレイアウトとインデックス計算
- ベースライン実装の最適化

**対象読者:**
- Jacobi法の基礎を学びたい方
- ベースライン実装を理解したい方
- アルゴリズムの詳細を確認したい方

---

## ドキュメントの使い方

### 初めての方向け
1. **SINGLE_THREAD_FLOW.md** - Jacobi法の基礎を理解
2. **BENCHMARKING_GUIDE.md** - ベンチマークの実行方法を学習
3. **IMPLEMENTATION_COMPARISON.md** - 各実装の違いを理解

### 実装を理解したい方向け
1. **SINGLE_THREAD_FLOW.md** - ベースライン実装
2. **IMPLEMENTATION_COMPARISON.md** - 並列化手法の比較
3. ソースコード（`rust/src/implementations/`）を参照

### ベンチマークを実行したい方向け
1. **BENCHMARKING_GUIDE.md** - 実行ガイドを確認
2. **BENCHMARK_STRATEGY.md** - 測定方法論を理解
3. `scripts/` ディレクトリのスクリプトを実行

### 結果を発表したい方向け
1. **PRESENTATION.md** - プレゼン資料のベース
2. **BENCHMARK_STRATEGY.md** - 測定の妥当性説明
3. `benchmark_results/` の結果データを参照

---

## 関連ドキュメント

プロジェクトルートにある主要ドキュメント:

### README.md
プロジェクト全体の概要と使い方

### CLAUDE.md
Claude Codeがプロジェクトを理解するための指示書（開発者向け）

**内容:**
- プロジェクト構造の説明
- ビルド・実行コマンド
- アーキテクチャの概要
- 重要な実装詳細

---

## その他のリソース

### ベンチマーク結果
`../benchmark_results/` ディレクトリ:
- 測定結果の生データ
- 分析レポート（Markdown）
- グラフ・図表（PNG）

### 解析ツール
`../python/` ディレクトリ:
- 結果可視化スクリプト
- テーブル生成ツール
- 統計分析ツール

### 実行スクリプト
`../scripts/` ディレクトリ:
- 正確性検証スクリプト
- ベンチマーク実行スクリプト
- 包括的測定スクリプト

---

## ドキュメントの更新について

ドキュメントは実装と同期させる必要があります:

### 実装を変更した場合
- **IMPLEMENTATION_COMPARISON.md** の該当セクションを更新
- **SINGLE_THREAD_FLOW.md** の基礎部分に影響がある場合は更新

### ベンチマーク手法を変更した場合
- **BENCHMARK_STRATEGY.md** の測定方法論を更新
- **BENCHMARKING_GUIDE.md** の実行手順を更新

### 新しい発見があった場合
- **PRESENTATION.md** に主要結果を追加
- 対応するドキュメントに詳細を記載

---

## ドキュメントのフォーマット

すべてのドキュメントはMarkdown形式（.md）で記述されており:
- GitHub上で読みやすく表示されます
- テキストエディタで簡単に編集できます
- 多くのMarkdownビューアで表示可能です

### 推奨ビューア
- GitHub（ブラウザ）
- VS Code（Markdown Preview機能）
- Typora、MacDown（専用Markdownエディタ）

---

## ファイル構成

```
docs/
├── README.md                        # このファイル
├── IMPLEMENTATION_COMPARISON.md     # 実装比較
├── BENCHMARK_STRATEGY.md            # ベンチマーク戦略
├── BENCHMARKING_GUIDE.md            # 実行ガイド
├── PRESENTATION.md                  # プレゼン資料
└── SINGLE_THREAD_FLOW.md            # シングルスレッドフロー
```
