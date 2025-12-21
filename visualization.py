import re
import matplotlib.pyplot as plt
import math
import os

# Data container
data = []

# File path
file_path = 'prod_result.txt'

# Parsing logic
current_grid_size = 0
grid_pattern = re.compile(r'(\d+)×(\d+)')

with open(file_path, 'r', encoding='utf-8') as f:
    lines = f.readlines()

for line in lines:
    line = line.strip()
    # Check for grid size header
    if "ベンチマーク設定" in line:
        match = grid_pattern.search(line)
        if match:
            # Assuming square grid
            current_grid_size = int(match.group(1))
        continue
    
    # Check for data lines
    # Method names typically start the line. 
    # Known methods: Single Thread, Barrier, Counter Sync, OpenMP/Rayon
    if any(m in line for m in ["Single Thread", "Barrier", "Counter Sync", "OpenMP/Rayon"]):
        parts = re.split(r'\s{2,}', line) # Split by 2 or more spaces
        if len(parts) >= 4:
            method_name = parts[0]
            try:
                c_time = float(parts[1])
                rust_safe_time = float(parts[2])
                rust_unsafe_time = float(parts[3])
                
                data.append({
                    'grid_size': current_grid_size,
                    'method': method_name,
                    'lang': 'C',
                    'time': c_time
                })
                data.append({
                    'grid_size': current_grid_size,
                    'method': method_name,
                    'lang': 'Rust Safe',
                    'time': rust_safe_time
                })
                data.append({
                    'grid_size': current_grid_size,
                    'method': method_name,
                    'lang': 'Rust Unsafe',
                    'time': rust_unsafe_time
                })
            except ValueError:
                continue

# Derived metrics (Throughput: Giga Updates Per Second)
# Updates = GridSize * GridSize * 1000 steps
for entry in data:
    total_updates = entry['grid_size'] * entry['grid_size'] * 1000
    entry['gups'] = (total_updates / entry['time']) / 1e9

# --- Plotting ---
os.makedirs('graphs', exist_ok=True)

# 1. Execution Time vs Grid Size (Log-Log)
methods = sorted(list(set(d['method'] for d in data)))
langs = ['C', 'Rust Safe', 'Rust Unsafe']
colors = {'C': 'blue', 'Rust Safe': 'green', 'Rust Unsafe': 'red'}
markers = {'C': 'o', 'Rust Safe': 's', 'Rust Unsafe': '^'}

fig, ax = plt.subplots(figsize=(10, 6))
for method in methods:
    for lang in langs:
        subset = [d for d in data if d['method'] == method and d['lang'] == lang]
        subset.sort(key=lambda x: x['grid_size'])
        if not subset: continue
        
        sizes = [d['grid_size'] for d in subset]
        times = [d['time'] for d in subset]
        
        ax.plot(sizes, times, marker=markers[lang], label=f"{method} - {lang}", linestyle='-' if method == 'OpenMP/Rayon' or method == 'Barrier' else '--')

ax.set_xlabel('Grid Size (N)')
ax.set_ylabel('Execution Time (s)')
ax.set_title('Execution Time vs Grid Size (Log-Log)')
ax.set_xscale('log')
ax.set_yscale('log')
ax.grid(True, which="both", ls="-")
# Legend might be too big, let's put it outside
ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
plt.tight_layout()
plt.savefig('graphs/time_vs_size_all.png')
plt.close()


# 2. Throughput (GUPS) vs Grid Size
fig, ax = plt.subplots(figsize=(10, 6))
for method in methods:
    for lang in langs:
        subset = [d for d in data if d['method'] == method and d['lang'] == lang]
        subset.sort(key=lambda x: x['grid_size'])
        if not subset: continue
        
        sizes = [d['grid_size'] for d in subset]
        gups = [d['gups'] for d in subset]
        # Use different line styles for methods to distinguish
        ls = '-'
        if method == 'Single Thread': ls = ':'
        elif method == 'Counter Sync': ls = '-.'
        
        ax.plot(sizes, gups, marker=markers[lang], label=f"{method} - {lang}", linestyle=ls, color=colors[lang])

ax.set_xlabel('Grid Size (N)')
ax.set_ylabel('Throughput (Giga Updates/sec)')
ax.set_title('Throughput vs Grid Size (Higher is Better)')
ax.set_xscale('log')
ax.grid(True)
ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
plt.tight_layout()
plt.savefig('graphs/throughput_vs_size.png')
plt.close()


# 3. Bar Chart for Largest Grid Size (2048)
target_size = 2048
subset_2048 = [d for d in data if d['grid_size'] == target_size]

if subset_2048:
    fig, ax = plt.subplots(figsize=(10, 6))
    
    # Organize data for bar chart
    # X axis: Method, Grouped by Lang
    import numpy as np
    
    methods_list = methods 
    x = np.arange(len(methods_list))
    width = 0.25
    
    for i, lang in enumerate(langs):
        times = []
        for m in methods_list:
            val = next((item['time'] for item in subset_2048 if item['method'] == m and item['lang'] == lang), 0)
            times.append(val)
        
        rects = ax.bar(x + i*width, times, width, label=lang, color=colors[lang])
        # Add labels
        ax.bar_label(rects, fmt='%.2f', padding=3)

    ax.set_xlabel('Method')
    ax.set_ylabel('Time (s)')
    ax.set_title(f'Execution Time by Method (Grid {target_size}x{target_size})')
    ax.set_xticks(x + width)
    ax.set_xticklabels(methods_list)
    ax.legend()
    ax.grid(axis='y', linestyle='--', alpha=0.7)
    
    plt.tight_layout()
    plt.savefig(f'graphs/bar_chart_{target_size}.png')
    plt.close()
    
# 4. Speedup vs Single Thread (C) for 2048
# Baseline: C Single Thread at 2048
baseline = next((d['time'] for d in subset_2048 if d['method'] == 'Single Thread' and d['lang'] == 'C'), None)

if baseline:
    fig, ax = plt.subplots(figsize=(10, 6))
    x = np.arange(len(methods_list))
    width = 0.25
    
    for i, lang in enumerate(langs):
        speedups = []
        for m in methods_list:
            val = next((item['time'] for item in subset_2048 if item['method'] == m and item['lang'] == lang), None)
            if val:
                speedups.append(baseline / val)
            else:
                speedups.append(0)
        
        rects = ax.bar(x + i*width, speedups, width, label=lang, color=colors[lang])
        ax.bar_label(rects, fmt='%.1fx', padding=3)

    ax.set_xlabel('Method')
    ax.set_ylabel('Speedup (vs C Single Thread)')
    ax.set_title(f'Speedup Relative to C Single Thread (Grid {target_size}x{target_size})')
    ax.set_xticks(x + width)
    ax.set_xticklabels(methods_list)
    ax.axhline(y=1.0, color='k', linestyle='-', alpha=0.3)
    ax.legend()
    ax.grid(axis='y', linestyle='--', alpha=0.7)
    
    plt.tight_layout()
    plt.savefig(f'graphs/speedup_{target_size}.png')
    plt.close()

print("Graphs generated in 'graphs/' directory.")
