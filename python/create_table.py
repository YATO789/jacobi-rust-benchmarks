#!/usr/bin/env python3
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

# 実装タイプのリスト
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

# Markdown形式の表を作成
output_lines = []
output_lines.append("# Jacobi Method Benchmark Results")
output_lines.append("")
output_lines.append("## Performance Comparison Table (Median Time in ms)")
output_lines.append("")

# ヘッダー行
header = "| Implementation | " + " | ".join([f"{size}x{size}" for size in grid_sizes]) + " |"
output_lines.append(header)

# セパレーター行
separator = "|" + "|".join([":---"] + ["---:" for _ in grid_sizes]) + "|"
output_lines.append(separator)

# データ行
for impl in implementations:
    row_data = [impl]
    for grid_size in grid_sizes:
        if impl in data[grid_size]:
            row_data.append(f"{data[grid_size][impl]:.2f}")
        else:
            row_data.append("N/A")
    output_lines.append("| " + " | ".join(row_data) + " |")

output_lines.append("")
output_lines.append("## Performance Improvement (vs Single Thread)")
output_lines.append("")

# ヘッダー行
output_lines.append(header)
output_lines.append(separator)

# 高速化率を計算
for impl in implementations:
    if "Single Thread" in impl:
        continue  # Skip single thread implementations

    row_data = [impl]
    for grid_size in grid_sizes:
        # 対応するSingle Threadの実装を特定
        if impl.startswith("C "):
            baseline_impl = "C Single Thread"
        else:
            baseline_impl = "Rust Single Thread"

        if impl in data[grid_size] and baseline_impl in data[grid_size]:
            speedup = data[grid_size][baseline_impl] / data[grid_size][impl]
            row_data.append(f"{speedup:.2f}x")
        else:
            row_data.append("N/A")
    output_lines.append("| " + " | ".join(row_data) + " |")

# ファイルに出力
output_path = "benchmark_results/benchmark_table.md"
with open(output_path, 'w', encoding='utf-8') as f:
    f.write("\n".join(output_lines))

print(f"Saved benchmark table: {output_path}")

# コンソールにも出力
print("\n" + "\n".join(output_lines))
