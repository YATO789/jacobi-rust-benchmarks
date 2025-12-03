#!/usr/bin/env python3
import matplotlib.pyplot as plt
import matplotlib
matplotlib.rcParams['font.sans-serif'] = ['Arial', 'Helvetica', 'DejaVu Sans']
matplotlib.rcParams['axes.unicode_minus'] = False
import re
from pathlib import Path

# データ構造: {grid_size: {implementation: median_time}}
data = {}

# ベンチマークファイルのリスト (2048x2048を除外)
benchmark_files = [
    "benchmark_results/benchmark_64x64_1000steps_20251203_120654.txt",
    "benchmark_results/benchmark_128x128_1000steps_20251203_120825.txt",
    "benchmark_results/benchmark_512x512_1000steps_20251203_121206.txt",
    "benchmark_results/benchmark_1024x1024_1000steps_20251203_120917.txt",
]

# グリッドサイズのマッピング (2048を除外)
grid_sizes = [64, 128, 512, 1024]

# 実装タイプのリスト（カテゴリー別に分類）
implementation_categories = {
    "Sequential": [
        "C Single Thread",
        "Rust Single Thread"
    ],
    "Barrier": [
        "C Barrier",
        "Rust Barrier",
        "Rust Barrier Unsafe"
    ],
    "Semaphore": [
        "C Safe Semaphore",
        "Rust Safe Semaphore",
        "Rust Unsafe Semaphore"
    ],
    "OpenMP/Rayon": [
        "C OpenMP",
        "Rust Rayon",
        "Rust Rayon Unsafe"
    ]
}

# 全実装タイプのフラットリスト
implementations = [
    "C Single Thread",
    "C Safe Semaphore",
    "C Barrier",
    "C OpenMP",
    "Rust Single Thread",
    "Rust Unsafe Semaphore",
    "Rust Safe Semaphore",
    "Rust Barrier",
    "Rust Barrier Unsafe",
    "Rust Rayon",
    "Rust Rayon Unsafe"
]

# データ構造を初期化
for grid_size in grid_sizes:
    data[grid_size] = {}

# ファイルを読み込んでデータを抽出
for idx, filepath in enumerate(benchmark_files):
    grid_size = grid_sizes[idx]

    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # C言語セクションとRustセクションを分離
        c_section = content.split("============================================\nC言語実装\n")[1].split("============================================\nRust実装\n")[0]
        rust_section = content.split("============================================\nRust実装\n")[1]

        # C言語実装の中央値を抽出
        c_patterns = {
            "C Single Thread": r"Single Thread:.*?中央値:\s*([\d.]+)\s*ms",
            "C Safe Semaphore": r"Safe Semaphore:.*?中央値:\s*([\d.]+)\s*ms",
            "C Barrier": r"Barrier:.*?中央値:\s*([\d.]+)\s*ms",
            "C OpenMP": r"OpenMP:.*?中央値:\s*([\d.]+)\s*ms",
        }

        for impl_name, pattern in c_patterns.items():
            match = re.search(pattern, c_section, re.DOTALL)
            if match:
                data[grid_size][impl_name] = float(match.group(1))

        # Rust実装の中央値を抽出
        rust_patterns = {
            "Rust Single Thread": r"Single Thread:.*?中央値:\s*([\d.]+)ms",
            "Rust Unsafe Semaphore": r"Unsafe Semaphore:.*?中央値:\s*([\d.]+)ms",
            "Rust Safe Semaphore": r"Safe Semaphore:.*?中央値:\s*([\d.]+)ms",
            "Rust Barrier": r"(?<!Unsafe )Barrier:.*?中央値:\s*([\d.]+)ms",
            "Rust Barrier Unsafe": r"Barrier Unsafe:.*?中央値:\s*([\d.]+)ms",
            "Rust Rayon": r"(?<!Unsafe )Rayon:.*?中央値:\s*([\d.]+)ms",
            "Rust Rayon Unsafe": r"Rayon Unsafe:.*?中央値:\s*([\d.]+)ms",
        }

        for impl_name, pattern in rust_patterns.items():
            match = re.search(pattern, rust_section, re.DOTALL)
            if match:
                data[grid_size][impl_name] = float(match.group(1))

    except FileNotFoundError:
        print(f"Warning: {filepath} not found, skipping...")
        continue
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        continue

# カテゴリーごとに色を設定
category_colors = {
    "Sequential": 'blue',
    "Barrier": 'green',
    "Semaphore": 'red',
    "OpenMP/Rayon": 'purple'
}

# 実装ごとのマーカースタイル（C言語とRustを区別）
marker_styles = {}
for category, impls in implementation_categories.items():
    for impl in impls:
        if impl.startswith("C "):
            marker_styles[impl] = 's'  # 四角
        elif "Unsafe" in impl:
            marker_styles[impl] = '^'  # 三角
        else:
            marker_styles[impl] = 'o'  # 丸

# カテゴリーごとにサブプロットを作成
fig, axes = plt.subplots(2, 2, figsize=(16, 12))
fig.suptitle('Jacobi Method Benchmark: Performance vs Grid Size (By Category)',
             fontsize=16, fontweight='bold')

axes = axes.flatten()

for idx, (category, impls) in enumerate(implementation_categories.items()):
    ax = axes[idx]

    for impl in impls:
        x_values = []
        y_values = []

        for grid_size in grid_sizes:
            if impl in data[grid_size]:
                x_values.append(grid_size)
                y_values.append(data[grid_size][impl])

        if x_values:  # データがある場合のみプロット
            ax.plot(x_values, y_values,
                   marker=marker_styles.get(impl, 'o'),
                   label=impl,
                   linewidth=2,
                   markersize=8)

    ax.set_xlabel('Grid Size (NxN)', fontsize=11)
    ax.set_ylabel('Time (ms)', fontsize=11)
    ax.set_title(category, fontsize=13, fontweight='bold')
    ax.legend(loc='upper left', fontsize=9)
    ax.grid(True, alpha=0.3)
    ax.set_xscale('log', base=2)
    ax.set_xticks(grid_sizes)
    ax.set_xticklabels([f'{size}x{size}' for size in grid_sizes], rotation=15)

plt.tight_layout()
plt.savefig('benchmark_results/performance_comparison_categorized.png', dpi=300, bbox_inches='tight')
print("Saved categorized graph: benchmark_results/performance_comparison_categorized.png")

# 全体を1つのグラフにも保存
fig2, ax2 = plt.subplots(figsize=(14, 8))

for category, impls in implementation_categories.items():
    for impl in impls:
        x_values = []
        y_values = []

        for grid_size in grid_sizes:
            if impl in data[grid_size]:
                x_values.append(grid_size)
                y_values.append(data[grid_size][impl])

        if x_values:
            ax2.plot(x_values, y_values,
                    marker=marker_styles.get(impl, 'o'),
                    label=impl,
                    linewidth=2,
                    markersize=6)

ax2.set_xlabel('Grid Size (NxN)', fontsize=12)
ax2.set_ylabel('Time (ms)', fontsize=12)
ax2.set_title('Jacobi Method Benchmark: Performance vs Grid Size (All Implementations)', fontsize=14, fontweight='bold')
ax2.legend(loc='upper left', fontsize=9, ncol=2)
ax2.grid(True, alpha=0.3)
ax2.set_xscale('log', base=2)
ax2.set_xticks(grid_sizes)
ax2.set_xticklabels([f'{size}x{size}' for size in grid_sizes])

plt.tight_layout()
plt.savefig('benchmark_results/performance_comparison_all.png', dpi=300, bbox_inches='tight')
print("Saved overall graph: benchmark_results/performance_comparison_all.png")

plt.show()
