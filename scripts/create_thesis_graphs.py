#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
卒業論文用グラフ生成スクリプト
prod_result.txtの結果から全ての有用なグラフを作成
"""

import matplotlib.pyplot as plt
import numpy as np
import os
from matplotlib import rcParams

# 日本語フォント設定（macOS/Linux）
rcParams['font.family'] = 'sans-serif'
rcParams['font.sans-serif'] = ['Hiragino Sans', 'Hiragino Kaku Gothic Pro', 'Yu Gothic', 'Meirio', 'DejaVu Sans']
rcParams['axes.unicode_minus'] = False

# 出力ディレクトリ作成
output_dir = 'thesis_graphs'
os.makedirs(output_dir, exist_ok=True)

# データ定義
grid_sizes = ['256×256', '512×512', '1024×1024', '2048×2048']
grid_sizes_num = [256, 512, 1024, 2048]
cells = [65536, 262144, 1048576, 4194304]
methods = ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']

# prod_result.txt からデータを読み込む関数
def load_data(filename):
    print(f"Reading data from {filename}...")
    new_data = {
        'C': {m: [0.0] * 4 for m in methods},
        'Rust Safe': {m: [0.0] * 4 for m in methods},
        'Rust Unsafe': {m: [0.0] * 4 for m in methods}
    }
    
    # サイズ文字列とインデックスのマッピング
    size_map = {
        '256×256': 0,
        '512×512': 1,
        '1024×1024': 2,
        '2048×2048': 3
    }
    
    current_idx = -1
    if not os.path.exists(filename):
        print(f"Error: {filename} not found.")
        return new_data

    with open(filename, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            # グリッドサイズの特定
            if 'ベンチマーク設定:' in line:
                for s, idx in size_map.items():
                    if s in line:
                        current_idx = idx
                        break
                continue
            
            # データの読み取り
            for method in methods:
                if line.startswith(method):
                    parts = line.split()
                    # 末尾3つの数値を取得 (C, Rust Safe, Rust Unsafe)
                    try:
                        vals = [float(v) for v in parts[-3:]]
                        if current_idx != -1:
                            new_data['C'][method][current_idx] = vals[0]
                            new_data['Rust Safe'][method][current_idx] = vals[1]
                            new_data['Rust Unsafe'][method][current_idx] = vals[2]
                    except (ValueError, IndexError):
                        pass
                    break
    return new_data

# プロジェクトルートの prod_result.txt パスを取得
script_dir = os.path.dirname(os.path.abspath(__file__))
project_root = os.path.dirname(script_dir)
result_path = os.path.join(project_root, 'prod_result.txt')

# データを読み込み
data = load_data(result_path)

# カラーマップ定義
colors = {
    'C': '#2E86AB',
    'Rust Safe': '#A23B72',
    'Rust Unsafe': '#F18F01'
}

method_colors = {
    'Single Thread': '#1f77b4',
    'Barrier': '#ff7f0e',
    'Counter Sync': '#2ca02c',
    'OpenMP/Rayon': '#d62728'
}

# ===========================
# グラフ1: グリッドサイズごとの実装比較（棒グラフ）
# ===========================
def create_grid_size_comparison():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    for idx, grid_size in enumerate(grid_sizes):
        ax = axes[idx]
        x = np.arange(len(methods))
        width = 0.3

        c_times = [data['C'][m][idx] for m in methods]
        rust_safe_times = [data['Rust Safe'][m][idx] for m in methods]
        rust_unsafe_times = [data['Rust Unsafe'][m][idx] for m in methods]

        rects1 = ax.bar(x - width, rust_safe_times, width, label='Rust Safe', color=colors['Rust Safe'], edgecolor='black', alpha=0.8)
        rects2 = ax.bar(x, rust_unsafe_times, width, label='Rust Unsafe', color=colors['Rust Unsafe'], edgecolor='black', alpha=0.8)
        rects3 = ax.bar(x + width, c_times, width, label='C', color=colors['C'], edgecolor='black', alpha=0.8)

        ax.set_xlabel('実装方法', fontsize=20,fontweight='bold')
        ax.set_ylabel('実行時間 (秒)', fontsize=20, fontweight='bold')
        ax.set_title(f'{grid_size} グリッド ({cells[idx]:,} 格子点)', fontsize=20, fontweight='bold')
        ax.set_xticks(x)

        # ラベルを短縮して回転なしで表示
        short_labels = [m.replace('OpenMP/Rayon', 'OMP/Rayon').replace('Single Thread', 'Single').replace('Counter Sync', 'Counter') for m in methods]
        ax.set_xticklabels(short_labels, fontsize=18, fontweight='bold')

        ax.tick_params(axis='y', labelsize=18)
        ax.legend(prop={'size': 16, 'weight': 'bold'})
        ax.grid(True, alpha=0.3, axis='y')
        ax.margins(y=0.15) # 上部にスペースを確保

        # バーの上に数値を表示
        def autolabel(rects):
            for rect in rects:
                height = rect.get_height()
                ax.text(rect.get_x() + rect.get_width()/2., height,
                        f'{height:.3f}',
                        ha='center', va='bottom', fontsize=10, fontweight='bold', rotation=0)

        autolabel(rects1)
        autolabel(rects2)
        autolabel(rects3)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/01_grid_size_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ1: グリッドサイズごとの実装比較")

# ===========================
# グラフ2: スケーラビリティ（実装方法別）
# ===========================
def create_scalability_plots():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    for idx, method in enumerate(methods):
        ax = axes[idx]

        for lang in ['C', 'Rust Safe', 'Rust Unsafe']:
            times = data[lang][method]
            ax.plot(grid_sizes_num, times, marker='o', linewidth=2,
                   markersize=8, label=lang, color=colors[lang])

        ax.set_xlabel('グリッドサイズ (N×N)', fontsize=12)
        ax.set_ylabel('実行時間 (秒)', fontsize=12)
        ax.set_title(f'{method}', fontsize=13, fontweight='bold')
        ax.set_xscale('log', base=2)
        ax.set_yscale('log')
        ax.legend(fontsize=10)
        ax.grid(True, which='both', alpha=0.3)
        ax.set_xticks(grid_sizes_num)
        ax.set_xticklabels(grid_sizes_num)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/02_scalability_by_method.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ2: スケーラビリティ（実装方法別）")

# ===========================
# グラフ3: スピードアップ率（Single Thread基準）
# ===========================
def create_speedup_plots():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    parallel_methods = ['Barrier', 'Counter Sync', 'OpenMP/Rayon']

    for idx, grid_size in enumerate(grid_sizes):
        ax = axes[idx]
        x = np.arange(len(parallel_methods))
        width = 0.25

        # Single Threadを基準としたスピードアップ計算
        c_speedup = [data['C']['Single Thread'][idx] / data['C'][m][idx] for m in parallel_methods]
        rust_safe_speedup = [data['Rust Safe']['Single Thread'][idx] / data['Rust Safe'][m][idx] for m in parallel_methods]
        rust_unsafe_speedup = [data['Rust Unsafe']['Single Thread'][idx] / data['Rust Unsafe'][m][idx] for m in parallel_methods]

        ax.bar(x - width, c_speedup, width, label='C', color=colors['C'])
        ax.bar(x, rust_safe_speedup, width, label='Rust Safe', color=colors['Rust Safe'])
        ax.bar(x + width, rust_unsafe_speedup, width, label='Rust Unsafe', color=colors['Rust Unsafe'])

        # 理想的なスピードアップ（2倍）を破線で表示
        ax.axhline(y=2.0, color='red', linestyle='--', linewidth=2, label='理想 (2×)')

        ax.set_xlabel('並列実装方法', fontsize=11)
        ax.set_ylabel('スピードアップ率', fontsize=11)
        ax.set_title(f'{grid_size} グリッド', fontsize=12, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels([m.replace('OpenMP/Rayon', 'OMP/Rayon') for m in parallel_methods], rotation=15, ha='right')
        ax.legend(fontsize=9)
        ax.grid(True, alpha=0.3, axis='y')
        ax.set_ylim(0, 2.5)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/03_speedup_ratio.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ3: スピードアップ率")

# ===========================
# グラフ5: 並列効率（Parallel Efficiency）
# ===========================
def create_parallel_efficiency():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    parallel_methods = ['Barrier', 'Counter Sync', 'OpenMP/Rayon']
    num_threads = 2  # 2スレッド並列

    for idx, grid_size in enumerate(grid_sizes):
        ax = axes[idx]
        x = np.arange(len(parallel_methods))
        width = 0.25

        # 並列効率 = スピードアップ / スレッド数 * 100%
        c_efficiency = [(data['C']['Single Thread'][idx] / data['C'][m][idx]) / num_threads * 100 for m in parallel_methods]
        rust_safe_efficiency = [(data['Rust Safe']['Single Thread'][idx] / data['Rust Safe'][m][idx]) / num_threads * 100 for m in parallel_methods]
        rust_unsafe_efficiency = [(data['Rust Unsafe']['Single Thread'][idx] / data['Rust Unsafe'][m][idx]) / num_threads * 100 for m in parallel_methods]

        ax.bar(x - width, c_efficiency, width, label='C', color=colors['C'])
        ax.bar(x, rust_safe_efficiency, width, label='Rust Safe', color=colors['Rust Safe'])
        ax.bar(x + width, rust_unsafe_efficiency, width, label='Rust Unsafe', color=colors['Rust Unsafe'])

        # 理想的な効率（100%）を破線で表示
        ax.axhline(y=100, color='red', linestyle='--', linewidth=2, label='理想 (100%)')

        ax.set_xlabel('並列実装方法', fontsize=11)
        ax.set_ylabel('並列効率 (%)', fontsize=11)
        ax.set_title(f'{grid_size} グリッド', fontsize=12, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels([m.replace('OpenMP/Rayon', 'OMP/Rayon') for m in parallel_methods], rotation=15, ha='right')
        ax.legend(fontsize=9)
        ax.grid(True, alpha=0.3, axis='y')
        ax.set_ylim(0, 120)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/05_parallel_efficiency.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ5: 並列効率")

# ===========================
# グラフ6: ヒートマップ（各言語）
# ===========================
def create_heatmaps():
    fig, axes = plt.subplots(1, 3, figsize=(20, 6))

    for idx, (lang, ax) in enumerate(zip(['C', 'Rust Safe', 'Rust Unsafe'], axes)):
        # データ行列作成（方法×グリッドサイズ）
        matrix = np.array([data[lang][m] for m in methods])

        im = ax.imshow(matrix, aspect='auto', cmap='YlOrRd', interpolation='nearest')

        # 軸設定
        ax.set_xticks(np.arange(len(grid_sizes)))
        ax.set_yticks(np.arange(len(methods)))
        ax.set_xticklabels(grid_sizes)
        ax.set_yticklabels(methods)

        # ラベル回転
        plt.setp(ax.get_xticklabels(), rotation=45, ha='right')

        # 値を表示
        for i in range(len(methods)):
            for j in range(len(grid_sizes)):
                text = ax.text(j, i, f'{matrix[i, j]:.3f}',
                             ha='center', va='center', color='black', fontsize=9)

        ax.set_title(f'{lang}', fontsize=14, fontweight='bold')
        ax.set_xlabel('グリッドサイズ', fontsize=12)
        ax.set_ylabel('実装方法', fontsize=12)

        # カラーバー
        cbar = plt.colorbar(im, ax=ax)
        cbar.set_label('実行時間 (秒)', fontsize=11)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/06_heatmap.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ6: ヒートマップ")

# ===========================
# グラフ7: 正規化グラフ（各サイズでの最速を1とする）
# ===========================
def create_normalized_plots():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    for idx, grid_size in enumerate(grid_sizes):
        ax = axes[idx]

        # 各グリッドサイズでの全実装の最小時間を取得
        all_times = []
        for lang in ['C', 'Rust Safe', 'Rust Unsafe']:
            for method in methods:
                all_times.append(data[lang][method][idx])
        min_time = min(all_times)

        x = np.arange(len(methods))
        width = 0.25

        # 正規化（最速を1とする）
        c_normalized = [data['C'][m][idx] / min_time for m in methods]
        rust_safe_normalized = [data['Rust Safe'][m][idx] / min_time for m in methods]
        rust_unsafe_normalized = [data['Rust Unsafe'][m][idx] / min_time for m in methods]

        ax.bar(x - width, c_normalized, width, label='C', color=colors['C'])
        ax.bar(x, rust_safe_normalized, width, label='Rust Safe', color=colors['Rust Safe'])
        ax.bar(x + width, rust_unsafe_normalized, width, label='Rust Unsafe', color=colors['Rust Unsafe'])

        ax.axhline(y=1.0, color='green', linestyle='--', linewidth=2, label='最速 (1.0)')

        ax.set_xlabel('実装方法', fontsize=11)
        ax.set_ylabel('相対実行時間（最速=1）', fontsize=11)
        ax.set_title(f'{grid_size} グリッド (最速: {min_time:.4f}秒)', fontsize=12, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels([m.replace('OpenMP/Rayon', 'OMP/Rayon') for m in methods], rotation=15, ha='right')
        ax.legend(fontsize=9)
        ax.grid(True, alpha=0.3, axis='y')

    plt.tight_layout()
    plt.savefig(f'{output_dir}/07_normalized_performance.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ7: 正規化グラフ")

# ===========================
# グラフ8: 総合パフォーマンス比較（全グリッドサイズの平均）
# ===========================
def create_overall_performance():
    fig, ax = plt.subplots(figsize=(12, 8))

    # 各言語×方法の平均実行時間を計算
    avg_times = {}
    for lang in ['C', 'Rust Safe', 'Rust Unsafe']:
        avg_times[lang] = [np.mean(data[lang][m]) for m in methods]

    x = np.arange(len(methods))
    width = 0.25

    ax.bar(x - width, avg_times['C'], width, label='C', color=colors['C'])
    ax.bar(x, avg_times['Rust Safe'], width, label='Rust Safe', color=colors['Rust Safe'])
    ax.bar(x + width, avg_times['Rust Unsafe'], width, label='Rust Unsafe', color=colors['Rust Unsafe'])

    ax.set_xlabel('実装方法', fontsize=13)
    ax.set_ylabel('平均実行時間 (秒)', fontsize=13)
    ax.set_title('総合パフォーマンス比較（全グリッドサイズの平均）', fontsize=15, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(methods)
    ax.legend(fontsize=12)
    ax.grid(True, alpha=0.3, axis='y')

    plt.tight_layout()
    plt.savefig(f'{output_dir}/08_overall_performance.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ8: 総合パフォーマンス比較")

# ===========================
# グラフ9: 計算量あたりの実行時間（セル数正規化）
# ===========================
def create_time_per_cell():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    # 時間ステップ数
    time_steps = 1000

    for idx, method in enumerate(methods):
        ax = axes[idx]

        for lang in ['C', 'Rust Safe', 'Rust Unsafe']:
            # 格子点×ステップあたりの時間（ナノ秒）
            time_per_cell_step = [(data[lang][method][i] / (cells[i] * time_steps)) * 1e9
                                  for i in range(len(cells))]
            ax.plot(grid_sizes_num, time_per_cell_step, marker='o', linewidth=2,
                   markersize=8, label=lang, color=colors[lang])

        ax.set_xlabel('グリッドサイズ (N×N)', fontsize=12)
        ax.set_ylabel('格子点・ステップあたりの時間 (ナノ秒)', fontsize=12)
        ax.set_title(f'{method}', fontsize=13, fontweight='bold')
        ax.set_xscale('log', base=2)
        ax.legend(fontsize=10)
        ax.grid(True, which='both', alpha=0.3)
        ax.set_xticks(grid_sizes_num)
        ax.set_xticklabels(grid_sizes_num)

    plt.tight_layout()
    plt.savefig(f'{output_dir}/09_time_per_cell.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ9: 計算量あたりの実行時間")

# ===========================
# グラフ10: Rust Safe vs Unsafe の性能差
# ===========================
def create_rust_safe_vs_unsafe():
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()


    for idx, method in enumerate(methods):
        ax = axes[idx]

        # Safe vs Unsafeの差分（%）
        diff_percent = []
        for i in range(len(grid_sizes)):
            safe_time = data['Rust Safe'][method][i]
            unsafe_time = data['Rust Unsafe'][method][i]
            diff = ((safe_time - unsafe_time) / unsafe_time) * 100
            diff_percent.append(diff)

        colors_diff = ['green' if d > 0 else 'red' for d in diff_percent]
        # 幅を0.6に設定（隙間を埋める）
        bars = ax.bar(grid_sizes, diff_percent, width=0.6, color=colors_diff, alpha=0.7, edgecolor='black')

        # 0%ラインを引く
        ax.axhline(y=0, color='black', linestyle='-', linewidth=1.5)

        # データ範囲に基づいてY軸のマージンを確保（ラベルが見切れないように）
        ax.margins(y=0.2)

        # 各バーに値を表示
        for bar, val in zip(bars, diff_percent):
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{val:.1f}%',
                   ha='center', va='bottom' if val > 0 else 'top', fontsize=18, fontweight='bold')

        ax.set_xlabel('グリッドサイズ', fontsize=22,fontweight='bold')
        ax.set_ylabel('性能差 (%)', fontsize=22,fontweight='bold')
        # サブプロットタイトルは簡潔に
        ax.set_title(f'{method}', fontsize=22, fontweight='bold')
        ax.grid(True, alpha=0.3, axis='y')
        plt.setp(ax.xaxis.get_majorticklabels(),fontsize=16, fontweight='bold')
        ax.tick_params(axis='y', labelsize=18)

        # 右上のサブプロット（idx=1）にグラフの読み方を説明
        if idx == 1:
            explanation_text ='正の値（緑）:Safeが遅い\n\n負の値（赤）:Unsafeが遅い'
            ax.text(0.52, 0.95, explanation_text,
                   transform=ax.transAxes,
                   fontsize=16, fontweight='bold',
                   verticalalignment='top',
                   horizontalalignment='left',
                   bbox=dict(boxstyle='round', facecolor='white', alpha=0.8, edgecolor='black', linewidth=1.0))

    # タイトル分のスペースを空けてレイアウト調整
    plt.tight_layout(rect=[0, 0, 1, 0.93])
    plt.savefig(f'{output_dir}/10_rust_safe_vs_unsafe.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("✓ グラフ10: Rust Safe vs Unsafe の性能差")

# ===========================
# 全グラフ生成実行
# ===========================
def main():
    print("\n" + "="*60)
    print("卒業論文用グラフ生成を開始します...")
    print("="*60 + "\n")

    create_grid_size_comparison()
    # create_scalability_plots()
    # create_speedup_plots()
    # create_parallel_efficiency()
    # create_heatmaps()
    # create_normalized_plots()
    # create_overall_performance()
    # create_time_per_cell()
    create_rust_safe_vs_unsafe()

    print("\n" + "="*60)
    print(f"✅ 全9種類のグラフを '{output_dir}/' に保存しました")
    print("="*60)
    print("\n生成されたグラフ:")
    print("  01. グリッドサイズごとの実装比較")
    print("  02. スケーラビリティ（実装方法別）")
    print("  03. スピードアップ率")
    print("  05. 並列効率")
    print("  06. ヒートマップ")
    print("  07. 正規化グラフ")
    print("  08. 総合パフォーマンス比較")
    print("  09. 計算量あたりの実行時間")
    print("  10. Rust Safe vs Unsafe の性能差")
    print("\n")

if __name__ == '__main__':
    main()
