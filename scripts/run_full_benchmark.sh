#!/bin/bash

# 完全ベンチマーク測定スクリプト
# 使用方法: ./run_full_benchmark.sh [quick|standard|full]

set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

MODE="${1:-standard}"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Jacobi法 包括的ベンチマーク${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "測定モード: ${GREEN}${MODE}${NC}"
echo ""

# プロジェクトルート（scriptsディレクトリ）
SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="$SCRIPTS_DIR/benchmark_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# サマリーファイル
SUMMARY_FILE="$RESULTS_DIR/summary_${MODE}_${TIMESTAMP}.csv"

echo "Grid_Size,Steps,Implementation,Min_ms,Median_ms,Mean_ms,Max_ms" > "$SUMMARY_FILE"

# 測定実行関数
run_measurement() {
    local size=$1
    local steps=$2

    echo -e "${GREEN}=== 測定中: ${size}x${size} グリッド, ${steps} ステップ ===${NC}"

    # ベンチマーク実行
    "${SCRIPTS_DIR}/run_benchmark.sh" -n ${size} -s ${steps} -c 5

    # 結果ファイルを探す
    RESULT_FILE=$(ls -t ${RESULTS_DIR}/benchmark_${size}x${size}_${steps}steps_*.txt 2>/dev/null | head -1)

    if [ -f "$RESULT_FILE" ]; then
        echo -e "${BLUE}結果ファイル: ${RESULT_FILE}${NC}"

        # 結果をCSVに抽出
        extract_results "$RESULT_FILE" ${size} ${steps}
    else
        echo -e "${RED}警告: 結果ファイルが見つかりません${NC}"
    fi

    # クールダウン
    echo -e "${YELLOW}クールダウン中...${NC}"
    sleep 10
}

# 結果抽出関数
extract_results() {
    local file=$1
    local size=$2
    local steps=$3

    # Pythonで結果を抽出
    python3 - "$file" ${size} ${steps} "$SUMMARY_FILE" << 'EOF'
import re
import sys

result_file = sys.argv[1]
grid_size = sys.argv[2]
time_steps = sys.argv[3]
summary_file = sys.argv[4]

with open(result_file, 'r', encoding='utf-8') as f:
    content = f.read()

# C言語セクションとRustセクションを分離
c_section = content.split('C言語実装')[1].split('Rust実装')[0] if 'C言語実装' in content else ''
rust_section = content.split('Rust実装')[1] if 'Rust実装' in content else ''

def extract_stats(section, lang_prefix):
    results = []
    current_impl = None
    stats = {}

    for line in section.split('\n'):
        # 実装名の検出
        if ':' in line and '試行' not in line and '最小値' not in line:
            potential_name = line.split(':')[0].strip()
            if potential_name and not potential_name.startswith('=') and len(potential_name) < 50:
                if current_impl and stats:
                    results.append((current_impl, stats))
                current_impl = potential_name
                stats = {}

        # 統計値の検出
        if current_impl:
            if '最小値' in line:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    val = float(match.group(1))
                    if match.group(2) == 's':
                        val *= 1000
                    stats['min'] = val
            elif '中央値' in line:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    val = float(match.group(1))
                    if match.group(2) == 's':
                        val *= 1000
                    stats['median'] = val
            elif '平均値' in line:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    val = float(match.group(1))
                    if match.group(2) == 's':
                        val *= 1000
                    stats['mean'] = val
            elif '最大値' in line:
                match = re.search(r'([0-9.]+)\s*(s|ms)', line)
                if match:
                    val = float(match.group(1))
                    if match.group(2) == 's':
                        val *= 1000
                    stats['max'] = val

    if current_impl and stats:
        results.append((current_impl, stats))

    return results

# 結果を抽出
c_results = extract_stats(c_section, 'C')
rust_results = extract_stats(rust_section, 'Rust')

# CSV書き込み
with open(summary_file, 'a') as f:
    for impl_name, stats in c_results:
        if all(k in stats for k in ['min', 'median', 'mean', 'max']):
            f.write(f"{grid_size},{time_steps},C_{impl_name},{stats['min']:.3f},{stats['median']:.3f},{stats['mean']:.3f},{stats['max']:.3f}\n")

    for impl_name, stats in rust_results:
        if all(k in stats for k in ['min', 'median', 'mean', 'max']):
            f.write(f"{grid_size},{time_steps},Rust_{impl_name},{stats['min']:.3f},{stats['median']:.3f},{stats['mean']:.3f},{stats['max']:.3f}\n")

print(f"結果を {summary_file} に追記しました")
EOF
}

# 測定セットの定義
case $MODE in
    quick)
        echo -e "${YELLOW}クイックモード: 基本性能確認（約30分）${NC}"
        echo ""

        run_measurement 1024 100
        run_measurement 2048 100
        ;;

    standard)
        echo -e "${YELLOW}標準モード: 主要パラメータ測定（約3時間）${NC}"
        echo ""

        # Phase 1: グリッドサイズ変化
        echo -e "${CYAN}Phase 1: グリッドサイズの影響${NC}"
        for SIZE in 512 1024 2048; do
            run_measurement ${SIZE} 100
        done

        # Phase 2: ステップ数変化
        echo -e "${CYAN}Phase 2: ステップ数の影響${NC}"
        for STEP in 50 100 200; do
            run_measurement 1024 ${STEP}
        done
        ;;

    full)
        echo -e "${YELLOW}フルモード: 完全測定（約8-10時間）${NC}"
        echo ""

        # Phase 1: グリッドサイズ sweep
        echo -e "${CYAN}Phase 1: グリッドサイズの影響${NC}"
        for SIZE in 256 512 1024 2048 4096; do
            run_measurement ${SIZE} 100
        done

        # Phase 2: ステップ数 sweep
        echo -e "${CYAN}Phase 2: ステップ数の影響${NC}"
        for STEP in 10 50 100 200 500; do
            run_measurement 1024 ${STEP}
        done

        # Phase 3: マトリックス測定
        echo -e "${CYAN}Phase 3: 組み合わせ測定${NC}"
        for SIZE in 512 1024 2048; do
            for STEP in 50 100 200; do
                # すでに測定済みの組み合わせはスキップ
                if [ ${SIZE} -eq 1024 ] && ([ ${STEP} -eq 100 ] || [ ${STEP} -eq 50 ] || [ ${STEP} -eq 200 ]); then
                    echo -e "${YELLOW}スキップ: ${SIZE}x${SIZE}, ${STEP}ステップ（既測定）${NC}"
                    continue
                fi
                if [ ${SIZE} -eq 512 ] && [ ${STEP} -eq 100 ]; then
                    echo -e "${YELLOW}スキップ: ${SIZE}x${SIZE}, ${STEP}ステップ（既測定）${NC}"
                    continue
                fi
                if [ ${SIZE} -eq 2048 ] && [ ${STEP} -eq 100 ]; then
                    echo -e "${YELLOW}スキップ: ${SIZE}x${SIZE}, ${STEP}ステップ（既測定）${NC}"
                    continue
                fi

                run_measurement ${SIZE} ${STEP}
            done
        done
        ;;

    *)
        echo -e "${RED}エラー: 無効なモード: ${MODE}${NC}"
        echo "使用方法: $0 [quick|standard|full]"
        exit 1
        ;;
esac

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}測定完了！${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "サマリーファイル: ${BLUE}${SUMMARY_FILE}${NC}"
echo ""
echo -e "${YELLOW}次のステップ:${NC}"
echo "1. 結果を確認: cat ${SUMMARY_FILE}"
echo "2. グラフ化: python3 visualize_results.py ${SUMMARY_FILE}"
echo ""
