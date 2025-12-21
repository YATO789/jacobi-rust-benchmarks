#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Rust Safe vs Unsafe の実測性能差分析
prod_result.txtのデータから実際の性能差を計算
"""

import matplotlib.pyplot as plt
import numpy as np
from matplotlib import rcParams

# 日本語フォント設定
rcParams['font.family'] = 'sans-serif'
rcParams['font.sans-serif'] = ['Hiragino Sans', 'Hiragino Kaku Gothic Pro', 'Yu Gothic', 'Meirio', 'DejaVu Sans']
rcParams['axes.unicode_minus'] = False

# データ定義
grid_sizes = ['128×128', '256×256', '512×512', '1024×1024', '2048×2048']
methods = ['Single Thread', 'Barrier', 'Counter Sync', 'OpenMP/Rayon']

# データ（秒）
data = {
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

print("="*80)
print("Rust Safe vs Unsafe 実測性能差分析")
print("="*80)
print()

# 各実装ごとの性能差を計算
for method in methods:
    print(f"【{method}】")
    print(f"{'グリッドサイズ':<15} {'Safe (秒)':<12} {'Unsafe (秒)':<12} {'差分 (秒)':<12} {'改善率 (%)':<12} {'判定'}")
    print("-" * 80)

    improvements = []

    for i, grid_size in enumerate(grid_sizes):
        safe_time = data['Rust Safe'][method][i]
        unsafe_time = data['Rust Unsafe'][method][i]
        diff = safe_time - unsafe_time
        improvement = ((safe_time - unsafe_time) / safe_time) * 100
        improvements.append(improvement)

        # 判定
        if improvement > 0:
            judgment = f"Unsafe が {improvement:.1f}% 高速"
        elif improvement < 0:
            judgment = f"Safe が {abs(improvement):.1f}% 高速"
        else:
            judgment = "同等"

        print(f"{grid_size:<15} {safe_time:<12.6f} {unsafe_time:<12.6f} {diff:<12.6f} {improvement:<12.2f} {judgment}")

    avg_improvement = np.mean(improvements)
    min_improvement = np.min(improvements)
    max_improvement = np.max(improvements)

    print(f"\n  平均改善率: {avg_improvement:+.2f}%")
    print(f"  最小改善率: {min_improvement:+.2f}% (グリッド: {grid_sizes[np.argmin(improvements)]})")
    print(f"  最大改善率: {max_improvement:+.2f}% (グリッド: {grid_sizes[np.argmax(improvements)]})")
    print()

# 全体統計
print("="*80)
print("【全体統計】")
print("="*80)

all_improvements = []
for method in methods:
    for i in range(len(grid_sizes)):
        safe_time = data['Rust Safe'][method][i]
        unsafe_time = data['Rust Unsafe'][method][i]
        improvement = ((safe_time - unsafe_time) / safe_time) * 100
        all_improvements.append(improvement)

print(f"全ケース平均改善率: {np.mean(all_improvements):+.2f}%")
print(f"標準偏差: {np.std(all_improvements):.2f}%")
print(f"中央値: {np.median(all_improvements):+.2f}%")
print(f"最小改善率: {np.min(all_improvements):+.2f}%")
print(f"最大改善率: {np.max(all_improvements):+.2f}%")
print()

# Unsafeが遅いケースを抽出
print("="*80)
print("【注意: Unsafeが遅くなっているケース】")
print("="*80)
for method in methods:
    for i, grid_size in enumerate(grid_sizes):
        safe_time = data['Rust Safe'][method][i]
        unsafe_time = data['Rust Unsafe'][method][i]
        improvement = ((safe_time - unsafe_time) / safe_time) * 100

        if improvement < 0:
            print(f"{method:<20} {grid_size:<15} Unsafeが {abs(improvement):.2f}% 遅い")

print()
print("="*80)
print("【結論】")
print("="*80)
print(f"• Unsafe版の性能改善範囲: {np.min(all_improvements):.2f}% ～ {np.max(all_improvements):.2f}%")
print(f"• 平均的な改善率: {np.mean(all_improvements):+.2f}%")

if np.mean(all_improvements) > 0:
    print(f"• Unsafe版は平均的に約 {np.mean(all_improvements):.1f}% 高速")
else:
    print(f"• Safe版は平均的に約 {abs(np.mean(all_improvements)):.1f}% 高速")

print(f"• ただし、{sum(1 for x in all_improvements if x < 0)} ケース ({sum(1 for x in all_improvements if x < 0)/len(all_improvements)*100:.1f}%) でUnsafeが逆に遅い")
print()
