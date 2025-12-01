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

# プロジェクトルート
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
C_DIR="$PROJECT_ROOT/c"
RUST_DIR="$PROJECT_ROOT/rust"

# 結果ディレクトリ
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULT_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.txt"

# クールダウン時間（秒）
COOLDOWN=5

# ベンチマーク設定
ITERATIONS=15
WARMUP=3

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Jacobi法 ベンチマーク比較ツール${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "プロジェクトルート: ${BLUE}$PROJECT_ROOT${NC}"
echo -e "結果保存先: ${BLUE}$RESULT_FILE${NC}"
echo -e "測定回数: ${YELLOW}$ITERATIONS${NC} (ウォームアップ: $WARMUP)"
echo -e "クールダウン: ${YELLOW}${COOLDOWN}秒${NC}"
echo ""

# 結果ディレクトリ作成
mkdir -p "$RESULTS_DIR"

# システム情報収集
echo -e "${GREEN}[1/5] システム情報を収集中...${NC}"

# グリッドサイズを取得
GRID_N=$(grep "pub const N:" "$RUST_DIR/src/grid.rs" | sed -E 's/.*= ([0-9]+).*/\1/' || echo "Unknown")
GRID_M=$(grep "pub const M:" "$RUST_DIR/src/grid.rs" | sed -E 's/.*= ([0-9]+).*/\1/' || echo "Unknown")
TIME_STEPS_VAL=$(grep "pub const TIME_STEPS:" "$RUST_DIR/src/grid.rs" | sed -E 's/.*= ([0-9]+).*/\1/' || echo "Unknown")

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
    echo "グリッドサイズ: ${GRID_N} × ${GRID_M}"
    echo "総セル数: $((GRID_N * GRID_M))"
    echo "TIME_STEPS: ${TIME_STEPS_VAL}"
    echo "測定回数: ${ITERATIONS}"
    echo "ウォームアップ: ${WARMUP}"
    echo ""
} > "$RESULT_FILE"

# ビルド
echo -e "${GREEN}[2/5] ビルド中...${NC}"
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
echo -e "${GREEN}[3/5] C版ベンチマーク実行中...${NC}"
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
echo -e "${GREEN}[4/5] Rust版ベンチマーク実行中...${NC}"
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
echo -e "${GREEN}[5/5] 結果を分析中...${NC}"
echo ""

# 結果ファイルから統計を抽出して比較表を作成
python3 - << 'EOF'
import re
import sys

result_file = sys.argv[1] if len(sys.argv) > 1 else None
if not result_file:
    sys.exit(1)

with open(result_file, 'r', encoding='utf-8') as f:
    content = f.read()

# 実装名と中央値を抽出
pattern = r'([^:\n]+):\s+.*?中央値:\s+([0-9.]+)\s*(s|ms)'
matches = re.findall(pattern, content, re.MULTILINE | re.DOTALL)

# C版とRust版で分ける
c_results = {}
rust_results = {}

in_c_section = False
in_rust_section = False

for line in content.split('\n'):
    if 'C言語実装' in line:
        in_c_section = True
        in_rust_section = False
    elif 'Rust実装' in line:
        in_c_section = False
        in_rust_section = True

    match = re.search(r'([^:\n]+):\s+.*?中央値:\s+([0-9.]+)\s*(s|ms)', line + '\n' + content[content.find(line):content.find(line)+500])
    if match:
        name = match.group(1).strip()
        value = float(match.group(2))
        unit = match.group(3)

        # ms -> s に統一
        if unit == 'ms':
            value = value / 1000.0

        if in_c_section and name not in c_results:
            c_results[name] = value
        elif in_rust_section and name not in rust_results:
            rust_results[name] = value

# より詳細な抽出
c_section = content.split('C言語実装')[1].split('Rust実装')[0] if 'C言語実装' in content else ''
rust_section = content.split('Rust実装')[1] if 'Rust実装' in content else ''

for section, results_dict in [(c_section, c_results), (rust_section, rust_results)]:
    current_impl = None
    for line in section.split('\n'):
        # 実装名の検出
        if ':' in line and '試行' not in line and '最小値' not in line and '中央値' not in line and '平均値' not in line and '最大値' not in line:
            potential_name = line.split(':')[0].strip()
            if potential_name and not potential_name.startswith('=') and len(potential_name) < 50:
                current_impl = potential_name
        # 中央値の検出
        elif '中央値' in line and current_impl:
            match = re.search(r'([0-9.]+)\s*(s|ms)', line)
            if match:
                value = float(match.group(1))
                unit = match.group(2)
                if unit == 'ms':
                    value = value / 1000.0
                if current_impl not in results_dict:
                    results_dict[current_impl] = value

# 結果を表示
print("\n" + "="*80)
print("ベンチマーク結果比較 (中央値)")
print("="*80)
print(f"{'実装名':<25} {'C (秒)':<15} {'Rust (秒)':<15} {'比較 (C/Rust)':<15}")
print("-"*80)

# 実装名のマッピング（表示順序を制御）
impl_order = [
    "Single Thread",
    "Unsafe Semaphore",
    "Safe Semaphore",
    "Barrier",
    "Rayon",
    "Channel",
    "unsafe parallel"
]

for impl_name in impl_order:
    c_val = c_results.get(impl_name)
    rust_val = rust_results.get(impl_name)

    if c_val is not None and rust_val is not None:
        ratio = c_val / rust_val
        ratio_str = f"{ratio:.2f}x"
        c_str = f"{c_val:.6f}"
        rust_str = f"{rust_val:.6f}"
        print(f"{impl_name:<25} {c_str:<15} {rust_str:<15} {ratio_str:<15}")
    elif c_val is not None:
        print(f"{impl_name:<25} {c_val:.6f}{'':<15} {'N/A':<15}")
    elif rust_val is not None:
        print(f"{impl_name:<25} {'N/A':<15} {rust_val:.6f}{'':<15}")

print("="*80)
print("\n注: 比較値はC実行時間 / Rust実行時間を表示")
print("    1.0より大きい = Rustの方が速い")
print("    1.0より小さい = Cの方が速い")
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

# 実装名と中央値を抽出
c_section = content.split('C言語実装')[1].split('Rust実装')[0] if 'C言語実装' in content else ''
rust_section = content.split('Rust実装')[1] if 'Rust実装' in content else ''

c_results = {}
rust_results = {}

for section, results_dict in [(c_section, c_results), (rust_section, rust_results)]:
    current_impl = None
    for line in section.split('\n'):
        if ':' in line and '試行' not in line and '最小値' not in line and '中央値' not in line and '平均値' not in line and '最大値' not in line:
            potential_name = line.split(':')[0].strip()
            if potential_name and not potential_name.startswith('=') and len(potential_name) < 50:
                current_impl = potential_name
        elif '中央値' in line and current_impl:
            match = re.search(r'([0-9.]+)\s*(s|ms)', line)
            if match:
                value = float(match.group(1))
                unit = match.group(2)
                if unit == 'ms':
                    value = value / 1000.0
                if current_impl not in results_dict:
                    results_dict[current_impl] = value

print(f"{'実装名':<25} {'C (秒)':<15} {'Rust (秒)':<15} {'比較 (C/Rust)':<15}")
print("-"*80)

impl_order = [
    "Single Thread",
    "Unsafe Semaphore",
    "Safe Semaphore",
    "Barrier",
    "Rayon",
    "Channel",
    "unsafe parallel"
]

for impl_name in impl_order:
    c_val = c_results.get(impl_name)
    rust_val = rust_results.get(impl_name)

    if c_val is not None and rust_val is not None:
        ratio = c_val / rust_val
        ratio_str = f"{ratio:.2f}x"
        c_str = f"{c_val:.6f}"
        rust_str = f"{rust_val:.6f}"
        print(f"{impl_name:<25} {c_str:<15} {rust_str:<15} {ratio_str:<15}")
    elif c_val is not None:
        print(f"{impl_name:<25} {c_val:.6f}{'':<15} {'N/A':<15}")
    elif rust_val is not None:
        print(f"{impl_name:<25} {'N/A':<15} {rust_val:.6f}{'':<15}")

print("="*80)
print("\n注: 比較値はC実行時間 / Rust実行時間を表示")
print("    1.0より大きい = Rustの方が速い")
print("    1.0より小さい = Cの方が速い")
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
