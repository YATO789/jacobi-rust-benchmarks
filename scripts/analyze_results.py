#!/usr/bin/env python3
"""
Jacobi法ベンチマーク結果の分析・可視化スクリプト

使用方法:
    python3 scripts/analyze_results.py prod_result.txt
"""

import re
import sys
from collections import defaultdict
import matplotlib.pyplot as plt
import matplotlib
matplotlib.rcParams['font.sans-serif'] = ['Arial Unicode MS', 'DejaVu Sans']
matplotlib.rcParams['axes.unicode_minus'] = False

def parse_benchmark_file(filename):
    """ベンチマーク結果ファイルをパース"""
    data = defaultdict(lambda: defaultdict(dict))

    with open(filename, 'r', encoding='utf-8') as f:
        content = f.read()

    # グリッドサイズとステップ数を抽出
    pattern = r'ベンチマーク設定: (\d+)×(\d+) グリッド.*?(\d+) ステップ.*?Method.*?C \(秒\).*?Rust Safe \(秒\).*?Rust Unsafe \(秒\).*?-+\s+(.*?)==='

    matches = re.findall(pattern, content, re.DOTALL)

    for match in matches:
        grid_size = int(match[0])
        steps = int(match[2])
        results_text = match[3]

        # 各メソッドの結果を抽出
        method_pattern = r'(.*?)\s+(\d+\.\d+)\s+(\d+\.\d+)\s+(\d+\.\d+)'
        methods = re.findall(method_pattern, results_text)

        for method_data in methods:
            method = method_data[0].strip()
            c_time = float(method_data[1])
            rust_safe_time = float(method_data[2])
            rust_unsafe_time = float(method_data[3])

            data[grid_size][method] = {
                'C': c_time,
                'Rust Safe': rust_safe_time,
                'Rust Unsafe': rust_unsafe_time
            }

    return data

def plot_execution_time(data):
    """実行時間の比較グラフ"""
    grid_sizes = sorted(data.keys())
    methods = ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']

    fig, axes = plt.subplots(2, 2, figsize=(15, 12))
    fig.suptitle('実行時間比較 (グリッドサイズ別)', fontsize=16, fontweight='bold')

    for idx, method in enumerate(methods):
        ax = axes[idx // 2, idx % 2]

        c_times = [data[size][method]['C'] for size in grid_sizes if method in data[size]]
        rust_safe_times = [data[size][method]['Rust Safe'] for size in grid_sizes if method in data[size]]
        rust_unsafe_times = [data[size][method]['Rust Unsafe'] for size in grid_sizes if method in data[size]]
        valid_sizes = [size for size in grid_sizes if method in data[size]]

        ax.plot(valid_sizes, c_times, 'o-', label='C', linewidth=2, markersize=8)
        ax.plot(valid_sizes, rust_safe_times, 's-', label='Rust Safe', linewidth=2, markersize=8)
        ax.plot(valid_sizes, rust_unsafe_times, '^-', label='Rust Unsafe', linewidth=2, markersize=8)

        ax.set_xlabel('グリッドサイズ', fontsize=12)
        ax.set_ylabel('実行時間 (秒)', fontsize=12)
        ax.set_title(method, fontsize=14, fontweight='bold')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_xscale('log', base=2)
        ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig('execution_time_comparison.png', dpi=300, bbox_inches='tight')
    print('✓ 保存: execution_time_comparison.png')

def plot_safety_overhead(data):
    """安全性オーバーヘッドのグラフ"""
    grid_sizes = sorted(data.keys())
    methods = ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']

    fig, ax = plt.subplots(figsize=(12, 7))

    for method in methods:
        overheads = []
        valid_sizes = []

        for size in grid_sizes:
            if method in data[size]:
                safe = data[size][method]['Rust Safe']
                unsafe = data[size][method]['Rust Unsafe']
                overhead = (safe - unsafe) / unsafe * 100
                overheads.append(overhead)
                valid_sizes.append(size)

        ax.plot(valid_sizes, overheads, 'o-', label=method, linewidth=2, markersize=8)

    ax.set_xlabel('グリッドサイズ', fontsize=12)
    ax.set_ylabel('安全性オーバーヘッド (%)', fontsize=12)
    ax.set_title('Rust Safe vs Unsafe: 安全性のコスト', fontsize=14, fontweight='bold')
    ax.legend()
    ax.grid(True, alpha=0.3)
    ax.set_xscale('log', base=2)
    ax.axhline(y=0, color='k', linestyle='--', alpha=0.3)

    plt.tight_layout()
    plt.savefig('safety_overhead.png', dpi=300, bbox_inches='tight')
    print('✓ 保存: safety_overhead.png')

def plot_language_comparison(data):
    """C vs Rust Unsafeの比較"""
    grid_sizes = sorted(data.keys())
    methods = ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']

    fig, ax = plt.subplots(figsize=(12, 7))

    for method in methods:
        ratios = []
        valid_sizes = []

        for size in grid_sizes:
            if method in data[size]:
                c_time = data[size][method]['C']
                rust_time = data[size][method]['Rust Unsafe']
                ratio = c_time / rust_time
                ratios.append(ratio)
                valid_sizes.append(size)

        ax.plot(valid_sizes, ratios, 'o-', label=method, linewidth=2, markersize=8)

    ax.set_xlabel('グリッドサイズ', fontsize=12)
    ax.set_ylabel('性能比 (C時間 / Rust Unsafe時間)', fontsize=12)
    ax.set_title('C vs Rust Unsafe: 言語間性能比較', fontsize=14, fontweight='bold')
    ax.legend()
    ax.grid(True, alpha=0.3)
    ax.set_xscale('log', base=2)
    ax.axhline(y=1.0, color='r', linestyle='--', alpha=0.5, label='同等性能')
    ax.fill_between([min(grid_sizes), max(grid_sizes)], 0.95, 1.05, alpha=0.2, color='green', label='±5%範囲')

    plt.tight_layout()
    plt.savefig('language_comparison.png', dpi=300, bbox_inches='tight')
    print('✓ 保存: language_comparison.png')

def plot_parallelization_efficiency(data):
    """並列化効率のグラフ"""
    grid_sizes = sorted(data.keys())
    languages = ['C', 'Rust Safe', 'Rust Unsafe']
    parallel_methods = ['Barrier', 'Counter Sync', 'OpenMP/Rayon']

    fig, axes = plt.subplots(1, 3, figsize=(18, 6))
    fig.suptitle('並列化効率 (2スレッド)', fontsize=16, fontweight='bold')

    for idx, lang in enumerate(languages):
        ax = axes[idx]

        for method in parallel_methods:
            efficiencies = []
            valid_sizes = []

            for size in grid_sizes:
                if method in data[size] and 'Single Thread' in data[size]:
                    single = data[size]['Single Thread'][lang]
                    parallel = data[size][method][lang]
                    efficiency = single / parallel / 2.0  # 2スレッド
                    efficiencies.append(efficiency * 100)  # パーセント表示
                    valid_sizes.append(size)

            ax.plot(valid_sizes, efficiencies, 'o-', label=method, linewidth=2, markersize=8)

        ax.set_xlabel('グリッドサイズ', fontsize=12)
        ax.set_ylabel('並列化効率 (%)', fontsize=12)
        ax.set_title(lang, fontsize=14, fontweight='bold')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_xscale('log', base=2)
        ax.axhline(y=100, color='r', linestyle='--', alpha=0.5, label='理想的効率')
        ax.set_ylim([0, 120])

    plt.tight_layout()
    plt.savefig('parallelization_efficiency.png', dpi=300, bbox_inches='tight')
    print('✓ 保存: parallelization_efficiency.png')

def print_summary_table(data):
    """サマリーテーブルの出力"""
    print("\n" + "="*80)
    print("ベンチマーク結果サマリー")
    print("="*80)

    for grid_size in sorted(data.keys()):
        print(f"\nグリッドサイズ: {grid_size}×{grid_size}")
        print("-" * 80)
        print(f"{'メソッド':<20} {'C (秒)':<12} {'Rust Safe':<12} {'Rust Unsafe':<12} {'オーバーヘッド'}")
        print("-" * 80)

        for method in ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']:
            if method in data[grid_size]:
                d = data[grid_size][method]
                overhead = (d['Rust Safe'] - d['Rust Unsafe']) / d['Rust Unsafe'] * 100
                print(f"{method:<20} {d['C']:<12.6f} {d['Rust Safe']:<12.6f} {d['Rust Unsafe']:<12.6f} {overhead:>6.1f}%")

def main():
    if len(sys.argv) < 2:
        print("使用方法: python3 scripts/analyze_results.py <結果ファイル>")
        print("例: python3 scripts/analyze_results.py prod_result.txt")
        sys.exit(1)

    filename = sys.argv[1]

    try:
        data = parse_benchmark_file(filename)

        if not data:
            print(f"エラー: {filename} からデータを読み取れませんでした")
            sys.exit(1)

        print(f"\n分析開始: {filename}")
        print(f"検出されたグリッドサイズ: {sorted(data.keys())}")

        # サマリーテーブル
        print_summary_table(data)

        # グラフ生成
        print("\nグラフ生成中...")
        plot_execution_time(data)
        plot_safety_overhead(data)
        plot_language_comparison(data)
        plot_parallelization_efficiency(data)

        print("\n✓ 分析完了！以下のファイルが生成されました:")
        print("  - execution_time_comparison.png")
        print("  - safety_overhead.png")
        print("  - language_comparison.png")
        print("  - parallelization_efficiency.png")

    except FileNotFoundError:
        print(f"エラー: ファイル '{filename}' が見つかりません")
        sys.exit(1)
    except Exception as e:
        print(f"エラー: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    main()
