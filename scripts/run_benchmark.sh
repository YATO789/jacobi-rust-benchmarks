#!/bin/bash

# 公平なベンチマーク実行スクリプト
# RustとCの実装を交互に実行して、システム負荷の影響を最小化

set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# デフォルト値
DEFAULT_GRID_SIZE=1000
DEFAULT_TIME_STEPS=100
DEFAULT_ITERATIONS=10
DEFAULT_WARMUP=5
DEFAULT_COOLDOWN=5

# 使用方法を表示
usage() {
    echo "使用方法: $0 [オプション]"
    echo ""
    echo "オプション:"
    echo "  -n, --grid-size N    グリッドサイズ (N×N) [デフォルト: $DEFAULT_GRID_SIZE]"
    echo "  -s, --steps NUM      計算ステップ数 [デフォルト: $DEFAULT_TIME_STEPS]"
    echo "  -i, --iterations NUM ベンチマーク測定回数 [デフォルト: $DEFAULT_ITERATIONS]"
    echo "  -w, --warmup NUM     ウォームアップ回数 [デフォルト: $DEFAULT_WARMUP]"
    echo "  -c, --cooldown SEC   クールダウン時間(秒) [デフォルト: $DEFAULT_COOLDOWN]"
    echo "  -h, --help           このヘルプを表示"
    echo ""
    echo "例:"
    echo "  $0 -n 500 -s 50              # 500×500グリッド, 50ステップ"
    echo "  $0 --grid-size 2000 --steps 200  # 2000×2000グリッド, 200ステップ"
    exit 0
}

# コマンドライン引数の解析
GRID_SIZE=$DEFAULT_GRID_SIZE
TIME_STEPS=$DEFAULT_TIME_STEPS
ITERATIONS=$DEFAULT_ITERATIONS
WARMUP=$DEFAULT_WARMUP
COOLDOWN=$DEFAULT_COOLDOWN

while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--grid-size)
            GRID_SIZE="$2"
            shift 2
            ;;
        -s|--steps)
            TIME_STEPS="$2"
            shift 2
            ;;
        -i|--iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        -w|--warmup)
            WARMUP="$2"
            shift 2
            ;;
        -c|--cooldown)
            COOLDOWN="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}エラー: 不明なオプション: $1${NC}"
            usage
            ;;
    esac
done

# プロジェクトルート（scriptsディレクトリの親ディレクトリ）
SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"
C_DIR="$PROJECT_ROOT/c"
RUST_DIR="$PROJECT_ROOT/rust"

# 結果ディレクトリ（scriptsディレクトリ内に作成）
RESULTS_DIR="$SCRIPTS_DIR/benchmark_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULT_FILE="$RESULTS_DIR/benchmark_${GRID_SIZE}x${GRID_SIZE}_${TIME_STEPS}steps_$TIMESTAMP.txt"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Jacobi法 ベンチマーク比較ツール${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "プロジェクトルート: ${BLUE}$PROJECT_ROOT${NC}"
echo -e "結果保存先: ${BLUE}$RESULT_FILE${NC}"
echo ""
echo -e "${YELLOW}ベンチマーク設定:${NC}"
echo -e "  グリッドサイズ: ${GREEN}${GRID_SIZE} × ${GRID_SIZE}${NC}"
echo -e "  計算ステップ数: ${GREEN}${TIME_STEPS}${NC}"
echo -e "  測定回数: ${GREEN}${ITERATIONS}${NC} (ウォームアップ: ${WARMUP})"
echo -e "  クールダウン: ${GREEN}${COOLDOWN}秒${NC}"
echo ""

# 結果ディレクトリ作成
mkdir -p "$RESULTS_DIR"

# システム情報収集
echo -e "${GREEN}[1/6] システム情報を収集中...${NC}"

{
    echo "============================================"
    echo "ベンチマーク実行情報"
    echo "============================================"
    echo "実行日時: $(date)"
    echo "ホスト: $(hostname)"
    echo "OS: $(uname -s) $(uname -r)"
    echo "CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Unknown')"
    echo "物理コア数: $(sysctl -n hw.physicalcpu 2>/dev/null || echo 'Unknown')"
    echo "論理コア数: $(sysctl -n hw.logicalcpu 2>/dev/null || echo 'Unknown')"
    echo "メモリ: $(sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 " GB"}' || echo 'Unknown')"
    echo ""
    echo "============================================"
    echo "ベンチマーク設定"
    echo "============================================"
    echo "グリッドサイズ: ${GRID_SIZE} × ${GRID_SIZE}"
    echo "総セル数: $((GRID_SIZE * GRID_SIZE))"
    echo "TIME_STEPS: ${TIME_STEPS}"
    echo "測定回数: ${ITERATIONS}"
    echo "ウォームアップ: ${WARMUP}"
    echo ""
} > "$RESULT_FILE"

# パラメータファイルを更新
echo -e "${GREEN}[2/6] パラメータを設定中...${NC}"

# Rust版のパラメータ更新
sed -i.bak "s/pub const N: usize = [0-9]*;/pub const N: usize = ${GRID_SIZE};/" "$RUST_DIR/src/grid.rs"
sed -i.bak "s/pub const M: usize = [0-9]*;/pub const M: usize = ${GRID_SIZE};/" "$RUST_DIR/src/grid.rs"
sed -i.bak "s/pub const TIME_STEPS: usize = [0-9]*;/pub const TIME_STEPS: usize = ${TIME_STEPS};/" "$RUST_DIR/src/grid.rs"
sed -i.bak "s/const BENCH_ITERATIONS: usize = [0-9]*;/const BENCH_ITERATIONS: usize = ${ITERATIONS};/" "$RUST_DIR/src/main.rs"
sed -i.bak "s/const BENCH_WARMUP: usize = [0-9]*;/const BENCH_WARMUP: usize = ${WARMUP};/" "$RUST_DIR/src/main.rs"

# C版のパラメータ更新
sed -i.bak "s/#define N [0-9]*/#define N ${GRID_SIZE}/" "$C_DIR/common/jacobi_common.h"
sed -i.bak "s/#define M [0-9]*/#define M ${GRID_SIZE}/" "$C_DIR/common/jacobi_common.h"
sed -i.bak "s/#define TIME_STEPS [0-9]*/#define TIME_STEPS ${TIME_STEPS}/" "$C_DIR/common/jacobi_common.h"
sed -i.bak "s/#define BENCH_ITERATIONS [0-9]*/#define BENCH_ITERATIONS ${ITERATIONS}/" "$C_DIR/main.c"
sed -i.bak "s/#define BENCH_WARMUP [0-9]*/#define BENCH_WARMUP ${WARMUP}/" "$C_DIR/main.c"

echo -e "${GREEN}  パラメータ設定完了${NC}"

# ビルド
echo -e "${GREEN}[3/6] ビルド中...${NC}"
echo "  - C版をビルド中..."
cd "$C_DIR"
make clean > /dev/null 2>&1
make > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}C版のビルドに失敗しました${NC}"
    exit 1
fi

echo "  - Rust版をビルド中..."
cd "$RUST_DIR"
cargo build --release > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}Rust版のビルドに失敗しました${NC}"
    exit 1
fi

echo -e "${GREEN}  ビルド完了${NC}"
echo ""

# 実行前のクールダウン
echo -e "${YELLOW}システムの安定化を待っています...${NC}"
sleep $COOLDOWN

# C版ベンチマーク実行
echo -e "${GREEN}[4/6] C版ベンチマーク実行中...${NC}"
cd "$C_DIR"
{
    echo "============================================"
    echo "C言語実装"
    echo "============================================"
    echo "コンパイラ: $(gcc --version | head -n 1)"
    echo "最適化フラグ: -O3"
    echo ""
} >> "$RESULT_FILE"

./jacobi_bench >> "$RESULT_FILE" 2>&1
echo -e "${GREEN}  C版完了${NC}"

# クールダウン
echo -e "${YELLOW}クールダウン中 (${COOLDOWN}秒)...${NC}"
sleep $COOLDOWN

# Rust版ベンチマーク実行
echo -e "${GREEN}[5/6] Rust版ベンチマーク実行中...${NC}"
cd "$RUST_DIR"
{
    echo ""
    echo "============================================"
    echo "Rust実装"
    echo "============================================"
    echo "コンパイラ: $(rustc --version)"
    echo "最適化レベル: release"
    echo ""
} >> "$RESULT_FILE"

cargo run --release 2>&1 | grep -v "Finished\|Running" >> "$RESULT_FILE"
echo -e "${GREEN}  Rust版完了${NC}"
echo ""

# 結果の分析と表示
echo -e "${GREEN}[6/6] 結果を分析中...${NC}"
echo ""

# 結果ファイルから統計を抽出して比較表を作成
python3 - "$RESULT_FILE" << 'EOF'
import re
import sys

result_file = sys.argv[1] if len(sys.argv) > 1 else None
if not result_file:
    print("エラー: 結果ファイルが指定されていません", file=sys.stderr)
    sys.exit(1)

with open(result_file, 'r', encoding='utf-8') as f:
    content = f.read()

# C版とRust版で分ける
c_results = {}
rust_results = {}

# C言語セクションとRustセクションを分離
c_section = content.split('C言語実装')[1].split('Rust実装')[0] if 'C言語実装' in content else ''
rust_section = content.split('Rust実装')[1] if 'Rust実装' in content else ''

# 新しいフォーマット: "実装名: min=0.123, avg=0.456, max=0.789" からデータを抽出
for section, results_dict in [(c_section, c_results), (rust_section, rust_results)]:
    for line in section.split('\n'):
        # 新しいフォーマットのパターン: "実装名: min=X, avg=Y, max=Z"
        match = re.search(r'^([^:]+):\s+min=[0-9.]+,\s+avg=([0-9.]+),\s+max=[0-9.]+', line.strip())
        if match:
            name = match.group(1).strip()
            avg_value = float(match.group(2))
            if name not in results_dict:
                results_dict[name] = avg_value

# 結果を表示
print("\n" + "="*80)
print("ベンチマーク結果比較 (平均値)")
print("="*80)
print(f"{'Method':<20} {'C (秒)':<15} {'Rust Safe (秒)':<20} {'Rust Unsafe (秒)':<20}")
print("-"*80)

# 実装名のマッピング（新形式: 縦軸=手法、横軸=言語）
# (手法名, C版名, Rust Safe名, Rust Unsafe名)
impl_matrix = [
    ("Single Thread", "Single Thread", "Single Thread", None),
    ("Barrier", "Barrier", "Barrier", "Barrier Unsafe"),
    ("Counter Sync", "Safe Semaphore", "Safe Semaphore", "Unsafe Semaphore"),
    ("OpenMP/Rayon", "OpenMP", "Rayon", "Rayon Unsafe"),
]

for method_name, c_name, rust_safe_name, rust_unsafe_name in impl_matrix:
    c_val = c_results.get(c_name) if c_name else None
    rust_safe_val = rust_results.get(rust_safe_name) if rust_safe_name else None
    rust_unsafe_val = rust_results.get(rust_unsafe_name) if rust_unsafe_name else None

    c_str = f"{c_val:.6f}" if c_val is not None else "N/A"
    safe_str = f"{rust_safe_val:.6f}" if rust_safe_val is not None else "N/A"
    unsafe_str = f"{rust_unsafe_val:.6f}" if rust_unsafe_val is not None else "N/A"

    print(f"{method_name:<20} {c_str:<15} {safe_str:<20} {unsafe_str:<20}")

print("="*80)
print()

EOF

# グリッド情報を含めて結果表示
python3 - "$RESULT_FILE" << 'DISPLAY_EOF'
import re
import sys

result_file = sys.argv[1]

with open(result_file, 'r', encoding='utf-8') as f:
    content = f.read()

# グリッド情報を抽出
grid_match = re.search(r'グリッドサイズ:\s*(\d+)\s*×\s*(\d+)', content)
steps_match = re.search(r'TIME_STEPS:\s*(\d+)', content)

if grid_match and steps_match:
    grid_n = int(grid_match.group(1))
    grid_m = int(grid_match.group(2))
    total_cells = grid_n * grid_m
    time_steps = int(steps_match.group(1))

    print("\n" + "="*80)
    print(f"ベンチマーク設定: {grid_n}×{grid_m} グリッド ({total_cells:,} セル), {time_steps} ステップ")
    print("="*80)

# C版とRust版で分ける
c_results = {}
rust_results = {}

# C言語セクションとRustセクションを分離
c_section = content.split('C言語実装')[1].split('Rust実装')[0] if 'C言語実装' in content else ''
rust_section = content.split('Rust実装')[1] if 'Rust実装' in content else ''

# 新しいフォーマット: "実装名: min=0.123, avg=0.456, max=0.789" からデータを抽出
for section, results_dict in [(c_section, c_results), (rust_section, rust_results)]:
    for line in section.split('\n'):
        # 新しいフォーマットのパターン: "実装名: min=X, avg=Y, max=Z"
        match = re.search(r'^([^:]+):\s+min=[0-9.]+,\s+avg=([0-9.]+),\s+max=[0-9.]+', line.strip())
        if match:
            name = match.group(1).strip()
            avg_value = float(match.group(2))
            if name not in results_dict:
                results_dict[name] = avg_value

# 実装名のマッピング（新形式: 縦軸=手法、横軸=言語）
# (手法名, C版名, Rust Safe名, Rust Unsafe名)
impl_matrix = [
    ("Single Thread", "Single Thread", "Single Thread", None),
    ("Barrier", "Barrier", "Barrier", "Barrier Unsafe"),
    ("Counter Sync", "Safe Semaphore", "Safe Semaphore", "Unsafe Semaphore"),
    ("OpenMP/Rayon", "OpenMP", "Rayon", "Rayon Unsafe"),
]

print(f"{'Method':<20} {'C (秒)':<15} {'Rust Safe (秒)':<20} {'Rust Unsafe (秒)':<20}")
print("-"*80)

for method_name, c_name, rust_safe_name, rust_unsafe_name in impl_matrix:
    c_val = c_results.get(c_name) if c_name else None
    rust_safe_val = rust_results.get(rust_safe_name) if rust_safe_name else None
    rust_unsafe_val = rust_results.get(rust_unsafe_name) if rust_unsafe_name else None

    c_str = f"{c_val:.6f}" if c_val is not None else "N/A"
    safe_str = f"{rust_safe_val:.6f}" if rust_safe_val is not None else "N/A"
    unsafe_str = f"{rust_unsafe_val:.6f}" if rust_unsafe_val is not None else "N/A"

    print(f"{method_name:<20} {c_str:<15} {safe_str:<20} {unsafe_str:<20}")

print("="*80)
print()

DISPLAY_EOF

# 完了メッセージ
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}ベンチマーク完了！${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "詳細な結果は以下に保存されました:"
echo -e "${BLUE}$RESULT_FILE${NC}"
echo ""
echo -e "結果を確認: ${YELLOW}cat $RESULT_FILE${NC}"
echo ""
