#!/usr/bin/env python3
import re
from pathlib import Path

# データ構造: {grid_size: {implementation: median_time}}
data = {}

# ベンチマークファイルのリスト
benchmark_files = [
    "benchmark_results/benchmark_64x64_1000steps_20251203_120654.txt",
    "benchmark_results/benchmark_128x128_1000steps_20251203_120825.txt",
    "benchmark_results/benchmark_512x512_1000steps_20251203_121206.txt",
    "benchmark_results/benchmark_1024x1024_1000steps_20251203_120917.txt",
]

# グリッドサイズのマッピング
grid_sizes = [64, 128, 512, 1024]

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
            "C Semaphore": r"Safe Semaphore:.*?中央値:\s*([\d.]+)\s*ms",
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
            "Rust Safe Barrier": r"(?<!Unsafe )Barrier:.*?中央値:\s*([\d.]+)ms",
            "Rust Unsafe Barrier": r"Barrier Unsafe:.*?中央値:\s*([\d.]+)ms",
            "Rust Safe Rayon": r"(?<!Unsafe )Rayon:.*?中央値:\s*([\d.]+)ms",
            "Rust Unsafe Rayon": r"Rayon Unsafe:.*?中央値:\s*([\d.]+)ms",
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

# 分析結果を作成
output_lines = []
output_lines.append("## 7. Results")
output_lines.append("")

# 各グリッドサイズについて分析
for grid_size in grid_sizes:
    output_lines.append(f"### Benchmark Results ({grid_size}x{grid_size} Grid, 1000 Steps)")
    output_lines.append("")
    output_lines.append("#### Comparison by Median (Unit: ms)")
    output_lines.append("")

    # テーブルヘッダー
    output_lines.append("| Implementation | C | Rust Safe | Rust Unsafe |")
    output_lines.append("|------|-------|-----------|-------------|")

    # Single Thread行
    c_single = data[grid_size].get("C Single Thread", None)
    rust_single = data[grid_size].get("Rust Single Thread", None)
    output_lines.append(f"| **Single Thread** | {c_single:.3f} | {rust_single:.3f} | - |")

    # Semaphore行
    c_sem = data[grid_size].get("C Semaphore", None)
    rust_safe_sem = data[grid_size].get("Rust Safe Semaphore", None)
    rust_unsafe_sem = data[grid_size].get("Rust Unsafe Semaphore", None)
    output_lines.append(f"| **Semaphore** | {c_sem:.3f} | {rust_safe_sem:.3f} | {rust_unsafe_sem:.3f} |")

    # Barrier行
    c_bar = data[grid_size].get("C Barrier", None)
    rust_safe_bar = data[grid_size].get("Rust Safe Barrier", None)
    rust_unsafe_bar = data[grid_size].get("Rust Unsafe Barrier", None)
    output_lines.append(f"| **Barrier** | {c_bar:.3f} | {rust_safe_bar:.3f} | {rust_unsafe_bar:.3f} |")

    # OpenMP/Rayon行
    c_omp = data[grid_size].get("C OpenMP", None)
    rust_safe_rayon = data[grid_size].get("Rust Safe Rayon", None)
    rust_unsafe_rayon = data[grid_size].get("Rust Unsafe Rayon", None)
    output_lines.append(f"| **OpenMP/Rayon** | {c_omp:.3f} | {rust_safe_rayon:.3f} | {rust_unsafe_rayon:.3f} |")

    output_lines.append("")
    output_lines.append("#### Key Findings")
    output_lines.append("")

    # 1. C vs Rust Safe Semaphore
    if c_sem and rust_safe_sem:
        ratio = rust_safe_sem / c_sem
        output_lines.append(f"**1. C vs Rust Safe Semaphore: C is {ratio:.2f}x faster**")
        output_lines.append(f"- C Semaphore: {c_sem:.3f} ms")
        output_lines.append(f"- Rust Safe Semaphore: {rust_safe_sem:.3f} ms")
        output_lines.append("- Reasons:")
        output_lines.append("  - Boundary buffer copy overhead")
        output_lines.append("  - Mutex lock/unlock cost")
        output_lines.append("  - Implementation strategy differences")
        output_lines.append("")

    # 2. Rust Safe vs Rust Unsafe
    if rust_safe_sem and rust_unsafe_sem:
        ratio = rust_safe_sem / rust_unsafe_sem
        output_lines.append(f"**2. Rust Safe vs Rust Unsafe Semaphore: Unsafe is {ratio:.2f}x faster**")
        output_lines.append(f"- Safe Semaphore: {rust_safe_sem:.3f} ms")
        output_lines.append(f"- Unsafe Semaphore: {rust_unsafe_sem:.3f} ms")
        output_lines.append("- Reasons:")
        output_lines.append("  - Elimination of bounds checking")
        output_lines.append("  - No boundary buffer needed (direct memory access)")
        output_lines.append("")

    # 3. Barrier comparison
    if rust_safe_bar and rust_unsafe_bar:
        ratio = rust_safe_bar / rust_unsafe_bar
        output_lines.append(f"**3. Barrier: Safe vs Unsafe - Unsafe is {ratio:.2f}x faster**")
        output_lines.append(f"- Rust Safe Barrier: {rust_safe_bar:.3f} ms")
        output_lines.append(f"- Rust Unsafe Barrier: {rust_unsafe_bar:.3f} ms")
        output_lines.append("- Simple synchronization pattern with lower overhead")
        output_lines.append("")

    # 4. Rayon comparison
    if rust_safe_rayon and rust_unsafe_rayon:
        ratio = rust_safe_rayon / rust_unsafe_rayon
        output_lines.append(f"**4. Rayon: Balanced performance - Unsafe is {ratio:.2f}x faster**")
        output_lines.append(f"- Rust Safe Rayon: {rust_safe_rayon:.3f} ms")
        output_lines.append(f"- Rust Unsafe Rayon: {rust_unsafe_rayon:.3f} ms")
        output_lines.append("- Practical performance despite high-level abstraction")
        output_lines.append("")

    # 5. OpenMP vs Rayon
    if c_omp and rust_safe_rayon:
        ratio = rust_safe_rayon / c_omp
        output_lines.append(f"**5. OpenMP vs Rayon: OpenMP is {ratio:.2f}x faster**")
        output_lines.append(f"- C OpenMP: {c_omp:.3f} ms")
        output_lines.append(f"- Rust Safe Rayon: {rust_safe_rayon:.3f} ms")
        output_lines.append("")

    output_lines.append("### Performance Ratio Analysis")
    output_lines.append("")

    # Safety Cost Analysis
    output_lines.append("#### Safety Cost")
    output_lines.append("```")
    output_lines.append("Safety Cost = Time(Rust Safe) / Time(Rust Unsafe)")
    output_lines.append("```")
    output_lines.append("")
    output_lines.append("| Implementation | Safety Cost | Notes |")
    output_lines.append("|------|-------------|------|")

    if rust_safe_sem and rust_unsafe_sem:
        safety_cost = rust_safe_sem / rust_unsafe_sem
        overhead_pct = (safety_cost - 1) * 100
        output_lines.append(f"| Semaphore | {safety_cost:.2f}x | Boundary buffer + Mutex ({overhead_pct:.0f}% overhead) |")

    if rust_safe_bar and rust_unsafe_bar:
        safety_cost = rust_safe_bar / rust_unsafe_bar
        overhead_pct = (safety_cost - 1) * 100
        output_lines.append(f"| Barrier | {safety_cost:.2f}x | Boundary buffer + Mutex ({overhead_pct:.0f}% overhead) |")

    if rust_safe_rayon and rust_unsafe_rayon:
        safety_cost = rust_safe_rayon / rust_unsafe_rayon
        overhead_pct = (safety_cost - 1) * 100
        output_lines.append(f"| Rayon | {safety_cost:.2f}x | Bounds checking ({overhead_pct:.0f}% overhead) |")

    output_lines.append("")

    min_cost = min(rust_safe_sem/rust_unsafe_sem if rust_safe_sem and rust_unsafe_sem else 999,
                   rust_safe_bar/rust_unsafe_bar if rust_safe_bar and rust_unsafe_bar else 999,
                   rust_safe_rayon/rust_unsafe_rayon if rust_safe_rayon and rust_unsafe_rayon else 999)
    max_cost = max(rust_safe_sem/rust_unsafe_sem if rust_safe_sem and rust_unsafe_sem else 0,
                   rust_safe_bar/rust_unsafe_bar if rust_safe_bar and rust_unsafe_bar else 0,
                   rust_safe_rayon/rust_unsafe_rayon if rust_safe_rayon and rust_unsafe_rayon else 0)
    output_lines.append(f"→ **Safety cost is {((min_cost - 1)*100):.0f}-{((max_cost - 1)*100):.0f}%**")
    output_lines.append("")

    # Language Overhead Analysis
    output_lines.append("#### Cross-Language Comparison")
    output_lines.append("```")
    output_lines.append("Language Overhead = Time(Rust Safe) / Time(C)")
    output_lines.append("```")
    output_lines.append("")
    output_lines.append("| Implementation | Overhead | Analysis |")
    output_lines.append("|------|----------|------|")

    if c_sem and rust_safe_sem:
        overhead = rust_safe_sem / c_sem
        output_lines.append(f"| Semaphore | {overhead:.2f}x | Implementation strategy difference is dominant |")

    if c_bar and rust_safe_bar:
        overhead = rust_safe_bar / c_bar
        output_lines.append(f"| Barrier | {overhead:.2f}x | Equivalent implementation strategy |")

    if c_single and rust_single:
        overhead = rust_single / c_single
        output_lines.append(f"| Single Thread | {overhead:.2f}x | Baseline comparison |")

    output_lines.append("")

    # Speedup Analysis
    output_lines.append("#### Parallel Speedup (vs Single Thread)")
    output_lines.append("")
    output_lines.append("| Implementation | C Speedup | Rust Safe Speedup | Rust Unsafe Speedup |")
    output_lines.append("|------|-----------|-------------------|---------------------|")

    # Semaphore speedup
    c_sem_speedup = c_single / c_sem if c_single and c_sem else 0
    rust_safe_sem_speedup = rust_single / rust_safe_sem if rust_single and rust_safe_sem else 0
    rust_unsafe_sem_speedup = rust_single / rust_unsafe_sem if rust_single and rust_unsafe_sem else 0
    output_lines.append(f"| Semaphore | {c_sem_speedup:.2f}x | {rust_safe_sem_speedup:.2f}x | {rust_unsafe_sem_speedup:.2f}x |")

    # Barrier speedup
    c_bar_speedup = c_single / c_bar if c_single and c_bar else 0
    rust_safe_bar_speedup = rust_single / rust_safe_bar if rust_single and rust_safe_bar else 0
    rust_unsafe_bar_speedup = rust_single / rust_unsafe_bar if rust_single and rust_unsafe_bar else 0
    output_lines.append(f"| Barrier | {c_bar_speedup:.2f}x | {rust_safe_bar_speedup:.2f}x | {rust_unsafe_bar_speedup:.2f}x |")

    # OpenMP/Rayon speedup
    c_omp_speedup = c_single / c_omp if c_single and c_omp else 0
    rust_safe_rayon_speedup = rust_single / rust_safe_rayon if rust_single and rust_safe_rayon else 0
    rust_unsafe_rayon_speedup = rust_single / rust_unsafe_rayon if rust_single and rust_unsafe_rayon else 0
    output_lines.append(f"| OpenMP/Rayon | {c_omp_speedup:.2f}x | {rust_safe_rayon_speedup:.2f}x | {rust_unsafe_rayon_speedup:.2f}x |")

    output_lines.append("")
    output_lines.append("---")
    output_lines.append("")

# ファイルに出力
output_path = "benchmark_results/analysis_results.md"
with open(output_path, 'w', encoding='utf-8') as f:
    f.write("\n".join(output_lines))

print(f"Saved analysis results: {output_path}")
