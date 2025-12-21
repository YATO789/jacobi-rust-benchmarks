#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
まとめグラフ生成スクリプト
1. 全実装の対数折れ線グラフ
2. グリッド毎の実装速度ランキング（横棒グラフ）
"""

import matplotlib.pyplot as plt
import numpy as np
import os
from matplotlib import rcParams

# 日本語フォント設定
rcParams['font.family'] = 'sans-serif'
rcParams['font.sans-serif'] = ['Hiragino Sans', 'Hiragino Kaku Gothic Pro', 'Yu Gothic', 'Meirio', 'DejaVu Sans']
rcParams['axes.unicode_minus'] = False

# 出力ディレクトリ
output_dir = 'thesis_graphs'
os.makedirs(output_dir, exist_ok=True)

# データ定義
grid_sizes = ['128×128', '256×256', '512×512', '1024×1024', '2048×2048']
grid_sizes_num = [128, 256, 512, 1024, 2048]
methods = ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']

# データ（秒）
data = {
    'C': {
        'Single Thread': [0.007381, 0.021216, 0.073577, 0.268410, 1.861033],
        'Barrier': [0.007350, 0.016303, 0.046838, 0.169496, 1.162719],
        'Counter Sync': [0.002448, 0.010163, 0.038945, 0.168335, 1.179004],
        'OpenMP/Rayon': [0.008559, 0.017101, 0.048930, 0.185734, 1.193724]
    },
    'Rust Safe': {
        'Single Thread': [0.005027, 0.019407, 0.074487, 0.310875, 2.096738],
        'Barrier': [0.008606, 0.017208, 0.047079, 0.187906, 1.278654],
        'Counter Sync': [0.002950, 0.010558, 0.041255, 0.176722, 1.262961],
        'OpenMP/Rayon': [0.008979, 0.018292, 0.054391, 0.198841, 1.260154]
    },
    'Rust Unsafe': {
        'Single Thread': [0.004788, 0.018822, 0.064871, 0.281307, 1.970121],
        'Barrier': [0.006936, 0.015019, 0.040949, 0.168604, 1.243680],
        'Counter Sync': [0.002332, 0.009652, 0.036015, 0.154532, 1.155940],
        'OpenMP/Rayon': [0.009755, 0.016735, 0.047668, 0.187984, 1.285493]
    }
}

# カラーマップ（12実装全て）
colors_combined = {
    'C - Single Thread': '#1f77b4',
    'C - Barrier': '#ff7f0e',
    'C - Counter Sync': '#2ca02c',
    'C - OpenMP/Rayon': '#d62728',
    'Rust Safe - Single Thread': '#9467bd',
    'Rust Safe - Barrier': '#8c564b',
    'Rust Safe - Counter Sync': '#e377c2',
    'Rust Safe - OpenMP/Rayon': '#7f7f7f',
    'Rust Unsafe - Single Thread': '#bcbd22',
    'Rust Unsafe - Barrier': '#17becf',
    'Rust Unsafe - Counter Sync': '#ff9896',
    'Rust Unsafe - OpenMP/Rayon': '#c5b0d5'
}

# 線種の定義
linestyles = {
    'Single Thread': '-',
    'Barrier': '--',
    'Counter Sync': '-.',
    'OpenMP/Rayon': ':'
}

# マーカーの定義
markers = {
    'C': 'o',
    'Rust Safe': 's',
    'Rust Unsafe': '^'
}

# ===========================
# グラフ13: 全実装の対数折れ線グラフ
# ===========================
def create_all_implementations_log_plot():
    fig, ax = plt.subplots(figsize=(16, 10))

    # 全12実装を描画
    for lang in ['C', 'Rust Safe', 'Rust Unsafe']:
        for method in methods:
            label = f'{lang} - {method}'
            times = data[lang][method]

            ax.plot(grid_sizes_num, times,
                   marker=markers[lang],
                   linestyle=linestyles[method],
                   linewidth=2.5,
                   markersize=9,
                   label=label,
                   color=colors_combined[label],
                   alpha=0.85)

    ax.set_xlabel('グリッドサイズ (N×N)', fontsize=14, fontweight='bold')
    ax.set_ylabel('実行時間 (秒)', fontsize=14, fontweight='bold')
    ax.set_title('全実装のスケーラビリティ比較（対数スケール）', fontsize=16, fontweight='bold')

    # 対数スケール設定
    ax.set_xscale('log', base=2)
    ax.set_yscale('log')

    # グリッド
    ax.grid(True, which='both', alpha=0.4, linestyle='-', linewidth=0.5)
    ax.grid(True, which='minor', alpha=0.2, linestyle=':', linewidth=0.5)

    # X軸の目盛り
    ax.set_xticks(grid_sizes_num)
    ax.set_xticklabels(grid_sizes_num, fontsize=11)

    # 凡例を2列で配置
    ax.legend(loc='upper left', fontsize=10, ncol=2, framealpha=0.95,
             edgecolor='black', fancybox=True)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/13_all_implementations_log.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ13: 全実装の対数折れ線グラフ")

# ===========================
# グラフ14: グリッド毎の実装速度ランキング（横棒グラフ）
# ===========================
def create_ranking_bar_charts():
    fig, axes = plt.subplots(5, 1, figsize=(14, 20))

    for idx, (grid_size, ax) in enumerate(zip(grid_sizes, axes)):
        # 全実装の時間を取得
        impl_times = []
        impl_labels = []

        for lang in ['C', 'Rust Safe', 'Rust Unsafe']:
            for method in methods:
                time = data[lang][method][idx]
                label = f'{lang} - {method}'
                impl_times.append(time)
                impl_labels.append(label)

        # 時間でソート（昇順 = 速い順）
        sorted_pairs = sorted(zip(impl_times, impl_labels))
        sorted_times, sorted_labels = zip(*sorted_pairs)

        # 色を設定（上位3つを強調）
        colors = []
        for i, label in enumerate(sorted_labels):
            if i == 0:  # 1位
                colors.append('#FFD700')  # ゴールド
            elif i == 1:  # 2位
                colors.append('#C0C0C0')  # シルバー
            elif i == 2:  # 3位
                colors.append('#CD7F32')  # ブロンズ
            else:
                colors.append(colors_combined[label])

        # 横棒グラフ
        y_pos = np.arange(len(sorted_labels))
        bars = ax.barh(y_pos, sorted_times, color=colors, alpha=0.85,
                       edgecolor='black', linewidth=1.2)

        # 各バーに時間を表示
        for i, (bar, time) in enumerate(zip(bars, sorted_times)):
            width = bar.get_width()
            # ランキング表示
            rank_text = f'#{i+1} '

            ax.text(width, bar.get_y() + bar.get_height()/2.,
                   f'  {rank_text}{time:.6f}秒',
                   ha='left', va='center', fontsize=9, fontweight='bold')

        # 軸設定
        ax.set_yticks(y_pos)
        ax.set_yticklabels(sorted_labels, fontsize=10)
        ax.invert_yaxis()  # 最速を上に
        ax.set_xlabel('実行時間 (秒)', fontsize=12, fontweight='bold')
        ax.set_title(f'{grid_size} グリッド - 実装速度ランキング',
                    fontsize=14, fontweight='bold')
        ax.grid(True, alpha=0.3, axis='x')

        # 最速を示す縦線
        fastest_time = sorted_times[0]
        ax.axvline(x=fastest_time, color='red', linestyle='--', linewidth=2, alpha=0.7)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/14_ranking_by_grid.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ14: グリッド毎の実装速度ランキング")

# ===========================
# 実行
# ===========================
def main():
    print("\n" + "="*60)
    print("まとめグラフ生成を開始します...")
    print("="*60 + "\n")

    create_all_implementations_log_plot()
    create_ranking_bar_charts()

    print("\n" + "="*60)
    print(f"✅ 2種類のまとめグラフを '{output_dir}/' に保存しました")
    print("="*60)
    print("\n生成されたグラフ:")
    print("  13. 全実装の対数折れ線グラフ")
    print("  14. グリッド毎の実装速度ランキング（横棒グラフ）")
    print("\n")

if __name__ == '__main__':
    main()
