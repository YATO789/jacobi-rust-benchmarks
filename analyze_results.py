#!/usr/bin/env python3
"""
ベンチマーク結果の詳細分析スクリプト
"""

import re
import sys
import os
from pathlib import Path
from datetime import datetime

def parse_benchmark_results(content):
    """ベンチマーク結果を解析"""

    # C版とRust版のセクションを分離
    sections = {'C': {}, 'Rust': {}}

    if 'C言語実装' not in content or 'Rust実装' not in content:
        return None

    c_section = content.split('C言語実装')[1].split('Rust実装')[0]
    rust_section = content.split('Rust実装')[1]

    for section_name, section_content in [('C', c_section), ('Rust', rust_section)]:
        current_impl = None
        impl_data = {}

        lines = section_content.split('\n')
        i = 0
        while i < len(lines):
            line = lines[i].strip()

            # 実装名の検出（":"で終わる行）
            if ':' in line and not any(keyword in line for keyword in
                ['試行', '最小値', '中央値', '平均値', '最大値', 'TIME_STEPS', '測定回数', 'スレッド数']):
                potential_name = line.split(':')[0].strip()
                if potential_name and not potential_name.startswith('=') and len(potential_name) < 50:
                    current_impl = potential_name
                    impl_data = {'trials': [], 'min': None, 'median': None, 'avg': None, 'max': None}

            # 試行データの検出
            elif '試行' in line and current_impl:
                match = re.search(r'試行\s+\d+:\s+([0-9.]+)\s*(s|ms)', line)
                if match:
                    value = float(match.group(1))
                    unit = match.group(2)
                    if unit == 'ms':
                        value = value / 1000.0
                    impl_data['trials'].append(value)

            # 統計値の検出
            elif '最小値' in line and current_impl:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    value = float(match.group(1))
                    if match.group(2) == 'ms':
                        value = value / 1000.0
                    impl_data['min'] = value

            elif '中央値' in line and current_impl:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    value = float(match.group(1))
                    if match.group(2) == 'ms':
                        value = value / 1000.0
                    impl_data['median'] = value

            elif '平均値' in line and current_impl:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    value = float(match.group(1))
                    if match.group(2) == 'ms':
                        value = value / 1000.0
                    impl_data['avg'] = value

            elif '最大値' in line and current_impl:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    value = float(match.group(1))
                    if match.group(2) == 'ms':
                        value = value / 1000.0
                    impl_data['max'] = value

                    # 統計完了、保存
                    if impl_data['median'] is not None:
                        sections[section_name][current_impl] = impl_data
                    current_impl = None

            i += 1

    return sections

def extract_benchmark_config(content):
    """ベンチマーク設定情報を抽出"""
    config = {}

    # グリッドサイズの抽出
    grid_match = re.search(r'グリッドサイズ:\s*(\d+)\s*×\s*(\d+)', content)
    if grid_match:
        config['grid_n'] = int(grid_match.group(1))
        config['grid_m'] = int(grid_match.group(2))
        config['total_cells'] = config['grid_n'] * config['grid_m']

    # TIME_STEPSの抽出
    steps_match = re.search(r'TIME_STEPS:\s*(\d+)', content)
    if steps_match:
        config['time_steps'] = int(steps_match.group(1))

    return config

def create_comparison_table(sections, config):
    """比較表を作成"""
    if not sections:
        return "データの解析に失敗しました"

    c_results = sections['C']
    rust_results = sections['Rust']

    # 実装の順序
    impl_order = [
        "Single Thread",
        "Unsafe Semaphore",
        "Safe Semaphore",
        "Barrier",
        "Rayon",
        "Channel",
        "unsafe parallel"
    ]

    output = []
    output.append("\n" + "="*100)
    output.append("詳細ベンチマーク比較")
    output.append("="*100)

    # ベンチマーク設定情報を表示
    if config:
        total_cells = config.get('total_cells', 'N/A')
        if isinstance(total_cells, int):
            total_cells_str = f"{total_cells:,}"
        else:
            total_cells_str = str(total_cells)
        output.append(f"グリッドサイズ: {config.get('grid_n', 'N/A')} × {config.get('grid_m', 'N/A')} "
                     f"(総セル数: {total_cells_str})")
        output.append(f"TIME_STEPS: {config.get('time_steps', 'N/A')}")
        output.append("="*100)
    output.append(f"{'実装名':<20} {'C中央値(s)':<12} {'C平均(s)':<12} {'Rust中央値(s)':<14} {'Rust平均(s)':<12} {'比較':<10} {'判定'}")
    output.append("-"*100)

    for impl_name in impl_order:
        c_data = c_results.get(impl_name)
        rust_data = rust_results.get(impl_name)

        if c_data and rust_data:
            c_median = c_data['median']
            c_avg = c_data['avg']
            rust_median = rust_data['median']
            rust_avg = rust_data['avg']

            ratio = c_median / rust_median

            if ratio > 1.1:
                verdict = "Rust勝利"
            elif ratio < 0.9:
                verdict = "C勝利"
            else:
                verdict = "同等"

            output.append(
                f"{impl_name:<20} "
                f"{c_median:>11.6f} "
                f"{c_avg:>11.6f} "
                f"{rust_median:>13.6f} "
                f"{rust_avg:>11.6f} "
                f"{ratio:>9.2f}x "
                f"{verdict}"
            )

    output.append("="*100)
    output.append("\n注: 比較 = C中央値 / Rust中央値")
    output.append("    > 1.1x : Rustが10%以上速い")
    output.append("    < 0.9x : Cが10%以上速い")
    output.append("    その他 : ほぼ同等")

    return '\n'.join(output)

def create_csv_output(sections, output_file):
    """CSV形式で結果を出力"""
    if not sections:
        return

    c_results = sections['C']
    rust_results = sections['Rust']

    impl_order = [
        "Single Thread",
        "Unsafe Semaphore",
        "Safe Semaphore",
        "Barrier",
        "Rayon",
        "Channel",
        "unsafe parallel"
    ]

    with open(output_file, 'w') as f:
        # ヘッダー
        f.write("Implementation,Language,Min(s),Median(s),Avg(s),Max(s),Trials\n")

        for impl_name in impl_order:
            # C版
            if impl_name in c_results:
                data = c_results[impl_name]
                trials = ';'.join([f"{t:.6f}" for t in data['trials']])
                f.write(f"{impl_name},C,{data['min']:.6f},{data['median']:.6f},"
                       f"{data['avg']:.6f},{data['max']:.6f},\"{trials}\"\n")

            # Rust版
            if impl_name in rust_results:
                data = rust_results[impl_name]
                trials = ';'.join([f"{t:.6f}" for t in data['trials']])
                f.write(f"{impl_name},Rust,{data['min']:.6f},{data['median']:.6f},"
                       f"{data['avg']:.6f},{data['max']:.6f},\"{trials}\"\n")

def main():
    if len(sys.argv) < 2:
        print("使用方法: python3 analyze_results.py <結果ファイル>")
        sys.exit(1)

    result_file = sys.argv[1]

    if not os.path.exists(result_file):
        print(f"エラー: ファイルが見つかりません: {result_file}")
        sys.exit(1)

    with open(result_file, 'r', encoding='utf-8') as f:
        content = f.read()

    # 解析
    sections = parse_benchmark_results(content)

    if not sections:
        print("エラー: ベンチマーク結果を解析できませんでした")
        sys.exit(1)

    # ベンチマーク設定を抽出
    config = extract_benchmark_config(content)

    # 比較表を表示
    print(create_comparison_table(sections, config))

    # CSV出力
    csv_file = result_file.replace('.txt', '.csv')
    create_csv_output(sections, csv_file)
    print(f"\nCSV形式で保存: {csv_file}")

if __name__ == '__main__':
    main()
