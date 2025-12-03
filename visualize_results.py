#!/usr/bin/env python3
"""
ベンチマーク結果の可視化スクリプト

使用方法:
    python3 visualize_results.py summary.csv
"""

import sys
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use('Agg')  # GUIなし環境対応
import numpy as np
from pathlib import Path

def load_data(csv_file):
    """CSVファイルを読み込む"""
    df = pd.read_csv(csv_file)
    print(f"データ読み込み完了: {len(df)} 行")
    print(f"カラム: {df.columns.tolist()}")
    print(f"\n実装一覧:")
    for impl in df['Implementation'].unique():
        print(f"  - {impl}")
    return df

def plot_grid_size_comparison(df, output_dir):
    """グリッドサイズ vs 実行時間"""
    # 固定ステップ数のデータを抽出
    steps_values = df['Steps'].unique()
    base_steps = 100 if 100 in steps_values else steps_values[0]

    data = df[df['Steps'] == base_steps].copy()

    if len(data) == 0:
        print(f"警告: Steps={base_steps}のデータがありません")
        return

    plt.figure(figsize=(12, 7))

    # 実装ごとにプロット
    for impl in data['Implementation'].unique():
        impl_data = data[data['Implementation'] == impl].sort_values('Grid_Size')
        plt.plot(impl_data['Grid_Size'], impl_data['Median_ms'],
                marker='o', label=impl, linewidth=2)

    plt.xlabel('Grid Size (N×N)', fontsize=12)
    plt.ylabel('Execution Time (ms)', fontsize=12)
    plt.title(f'Grid Size vs Performance (Steps={base_steps})', fontsize=14)
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.grid(True, alpha=0.3)
    plt.xscale('log', base=2)
    plt.yscale('log')
    plt.tight_layout()

    output_file = output_dir / 'grid_size_comparison.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"保存: {output_file}")
    plt.close()

def plot_time_steps_comparison(df, output_dir):
    """ステップ数 vs 実行時間"""
    # 固定グリッドサイズのデータを抽出
    grid_values = df['Grid_Size'].unique()
    base_grid = 1024 if 1024 in grid_values else grid_values[0]

    data = df[df['Grid_Size'] == base_grid].copy()

    if len(data) == 0:
        print(f"警告: Grid_Size={base_grid}のデータがありません")
        return

    plt.figure(figsize=(12, 7))

    # 実装ごとにプロット
    for impl in data['Implementation'].unique():
        impl_data = data[data['Implementation'] == impl].sort_values('Steps')
        plt.plot(impl_data['Steps'], impl_data['Median_ms'],
                marker='o', label=impl, linewidth=2)

    plt.xlabel('Time Steps', fontsize=12)
    plt.ylabel('Execution Time (ms)', fontsize=12)
    plt.title(f'Time Steps vs Performance (Grid={base_grid}×{base_grid})', fontsize=14)
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()

    output_file = output_dir / 'time_steps_comparison.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"保存: {output_file}")
    plt.close()

def plot_speedup_analysis(df, output_dir):
    """相対性能比較（Speedup）"""
    # ベースライン条件
    base_grid = 1024
    base_steps = 100

    data = df[(df['Grid_Size'] == base_grid) & (df['Steps'] == base_steps)].copy()

    if len(data) == 0:
        print(f"警告: ベースライン条件のデータがありません")
        return

    # C Single Threadをベースラインとする
    baseline_impl = 'C_Single Thread'
    baseline_data = data[data['Implementation'] == baseline_impl]

    if len(baseline_data) == 0:
        print(f"警告: ベースライン実装 {baseline_impl} が見つかりません")
        return

    baseline_time = baseline_data['Median_ms'].values[0]

    # Speedup計算
    speedups = []
    labels = []

    for impl in data['Implementation'].unique():
        impl_data = data[data['Implementation'] == impl]
        if len(impl_data) > 0:
            time = impl_data['Median_ms'].values[0]
            speedup = baseline_time / time
            speedups.append(speedup)
            labels.append(impl)

    # プロット
    plt.figure(figsize=(12, 8))
    colors = ['#2E86AB' if 'C_' in label else '#A23B72' if 'Safe' in label or 'Barrier' in label and 'Unsafe' not in label else '#F18F01' for label in labels]

    bars = plt.barh(labels, speedups, color=colors)
    plt.axvline(x=1.0, color='red', linestyle='--', linewidth=2, label='Baseline (C Single Thread)')
    plt.xlabel('Speedup (higher is better)', fontsize=12)
    plt.title(f'Relative Performance (Grid={base_grid}×{base_grid}, Steps={base_steps})', fontsize=14)
    plt.grid(True, alpha=0.3, axis='x')
    plt.legend()
    plt.tight_layout()

    output_file = output_dir / 'speedup_comparison.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"保存: {output_file}")
    plt.close()

def plot_safety_cost_heatmap(df, output_dir):
    """安全性コストのヒートマップ"""
    # Rust SafeとUnsafeを比較
    rust_safe_impls = [impl for impl in df['Implementation'].unique() if 'Rust_' in impl and 'Unsafe' not in impl and 'Single' not in impl]
    rust_unsafe_impls = [impl for impl in df['Implementation'].unique() if 'Unsafe' in impl]

    if len(rust_safe_impls) == 0 or len(rust_unsafe_impls) == 0:
        print("警告: SafeまたはUnsafe実装が見つかりません")
        return

    # 主要な実装ペアを選択
    safe_impl = 'Rust_Safe Semaphore' if 'Rust_Safe Semaphore' in rust_safe_impls else rust_safe_impls[0]
    unsafe_impl = 'Rust_Unsafe Semaphore' if 'Rust_Unsafe Semaphore' in rust_unsafe_impls else rust_unsafe_impls[0]

    # グリッドサイズとステップ数の組み合わせを抽出
    grid_sizes = sorted(df['Grid_Size'].unique())
    steps_list = sorted(df['Steps'].unique())

    # Safety Cost計算
    safety_costs = np.zeros((len(grid_sizes), len(steps_list)))

    for i, grid_size in enumerate(grid_sizes):
        for j, steps in enumerate(steps_list):
            safe_data = df[(df['Implementation'] == safe_impl) &
                          (df['Grid_Size'] == grid_size) &
                          (df['Steps'] == steps)]
            unsafe_data = df[(df['Implementation'] == unsafe_impl) &
                            (df['Grid_Size'] == grid_size) &
                            (df['Steps'] == steps)]

            if len(safe_data) > 0 and len(unsafe_data) > 0:
                safe_time = safe_data['Median_ms'].values[0]
                unsafe_time = unsafe_data['Median_ms'].values[0]
                safety_costs[i, j] = (safe_time / unsafe_time - 1.0) * 100  # パーセント表示
            else:
                safety_costs[i, j] = np.nan

    # ヒートマップ
    plt.figure(figsize=(10, 8))
    im = plt.imshow(safety_costs, cmap='RdYlGn_r', aspect='auto', interpolation='nearest')

    plt.colorbar(im, label='Safety Cost (%)')
    plt.xlabel('Time Steps', fontsize=12)
    plt.ylabel('Grid Size', fontsize=12)
    plt.title(f'Safety Cost Heatmap\n({safe_impl} vs {unsafe_impl})', fontsize=14)

    plt.xticks(range(len(steps_list)), steps_list)
    plt.yticks(range(len(grid_sizes)), [f'{s}²' for s in grid_sizes])

    # セル内に数値を表示
    for i in range(len(grid_sizes)):
        for j in range(len(steps_list)):
            if not np.isnan(safety_costs[i, j]):
                text = plt.text(j, i, f'{safety_costs[i, j]:.1f}%',
                              ha="center", va="center", color="black", fontsize=10)

    plt.tight_layout()

    output_file = output_dir / 'safety_cost_heatmap.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"保存: {output_file}")
    plt.close()

def generate_summary_table(df, output_dir):
    """サマリーテーブルを生成"""
    # ベースライン条件
    base_grid = 1024
    base_steps = 100

    data = df[(df['Grid_Size'] == base_grid) & (df['Steps'] == base_steps)].copy()

    if len(data) == 0:
        print(f"警告: ベースライン条件のデータがありません")
        return

    # テーブル作成
    summary = data[['Implementation', 'Min_ms', 'Median_ms', 'Mean_ms', 'Max_ms']].copy()
    summary = summary.sort_values('Median_ms')

    # Speedup追加
    baseline_time = summary.iloc[0]['Median_ms']
    summary['Speedup'] = baseline_time / summary['Median_ms']

    # ファイル出力
    output_file = output_dir / 'summary_table.txt'
    with open(output_file, 'w') as f:
        f.write(f"Performance Summary (Grid={base_grid}×{base_grid}, Steps={base_steps})\n")
        f.write("=" * 100 + "\n")
        f.write(summary.to_string(index=False))
        f.write("\n")

    print(f"保存: {output_file}")
    print("\n" + "="*100)
    print(summary.to_string(index=False))
    print("="*100)

def main():
    if len(sys.argv) < 2:
        print("使用方法: python3 visualize_results.py <csv_file>")
        sys.exit(1)

    csv_file = Path(sys.argv[1])
    if not csv_file.exists():
        print(f"エラー: ファイルが見つかりません: {csv_file}")
        sys.exit(1)

    # 出力ディレクトリ
    output_dir = csv_file.parent / 'visualizations'
    output_dir.mkdir(exist_ok=True)

    print(f"\n{'='*60}")
    print("Jacobi法ベンチマーク結果の可視化")
    print(f"{'='*60}\n")

    # データ読み込み
    df = load_data(csv_file)

    print("\n生成するグラフ:")
    print("1. グリッドサイズ vs 実行時間")
    plot_grid_size_comparison(df, output_dir)

    print("2. ステップ数 vs 実行時間")
    plot_time_steps_comparison(df, output_dir)

    print("3. 相対性能比較（Speedup）")
    plot_speedup_analysis(df, output_dir)

    print("4. 安全性コストのヒートマップ")
    plot_safety_cost_heatmap(df, output_dir)

    print("5. サマリーテーブル")
    generate_summary_table(df, output_dir)

    print(f"\n{'='*60}")
    print(f"可視化完了！")
    print(f"結果は {output_dir} に保存されました")
    print(f"{'='*60}\n")

if __name__ == '__main__':
    main()
