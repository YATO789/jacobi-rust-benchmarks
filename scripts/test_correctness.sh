#!/bin/bash

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"
C_DIR="$PROJECT_ROOT/c"
RUST_DIR="$PROJECT_ROOT/rust"
TEST_DIR="$SCRIPTS_DIR/test_results"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Jacobi法 結果一致性テスト${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# テスト結果ディレクトリ作成
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# パラメータを小さく設定してテスト（速度優先）
echo -e "${GREEN}[1/5] テスト用パラメータを設定中...${NC}"
GRID_SIZE=64
TEST_STEPS=100

# Rust版パラメータ更新
sed -i.bak "s/pub const N: usize = [0-9]*;/pub const N: usize = ${GRID_SIZE};/" "$RUST_DIR/src/grid.rs"
sed -i.bak "s/pub const M: usize = [0-9]*;/pub const M: usize = ${GRID_SIZE};/" "$RUST_DIR/src/grid.rs"

# C版パラメータ更新
sed -i.bak "s/#define N [0-9]*/#define N ${GRID_SIZE}/" "$C_DIR/common/jacobi_common.h"
sed -i.bak "s/#define M [0-9]*/#define M ${GRID_SIZE}/" "$C_DIR/common/jacobi_common.h"

echo -e "${GREEN}  グリッドサイズ: ${GRID_SIZE}x${GRID_SIZE}, ステップ数: ${TEST_STEPS}${NC}"
echo ""

# ビルド
echo -e "${GREEN}[2/5] ビルド中...${NC}"
echo "  - C版をビルド中..."
cd "$C_DIR"
make clean > /dev/null 2>&1
make test_output > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}C版のビルドに失敗しました${NC}"
    exit 1
fi

echo "  - Rust版をビルド中..."
cd "$RUST_DIR"
cargo build --release --bin test_output > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}Rust版のビルドに失敗しました${NC}"
    exit 1
fi
echo -e "${GREEN}  ビルド完了${NC}"
echo ""

# C版実行
echo -e "${GREEN}[3/5] C版テスト実行中...${NC}"
cd "$TEST_DIR"
"$C_DIR/test_output" > c_output.txt 2>&1
echo -e "${GREEN}  C版完了${NC}"
echo ""

# Rust版実行
echo -e "${GREEN}[4/5] Rust版テスト実行中...${NC}"
"$RUST_DIR/target/release/test_output" > rust_output.txt 2>&1
echo -e "${GREEN}  Rust版完了${NC}"
echo ""

# 結果比較
echo -e "${GREEN}[5/5] 結果を比較中...${NC}"
echo ""

# Python で比較スクリプトを実行
python3 - << 'EOF'
import struct
import os
import sys

def read_grid_file(filepath):
    """バイナリファイルからグリッドデータを読み込む"""
    with open(filepath, 'rb') as f:
        # ヘッダー読み込み
        n = struct.unpack('<I', f.read(4))[0]
        m = struct.unpack('<I', f.read(4))[0]

        # データ読み込み
        data = []
        for _ in range(n * m):
            val = struct.unpack('<d', f.read(8))[0]
            data.append(val)

        return n, m, data

def compare_grids(file1, file2, name1, name2, tolerance=1e-10):
    """2つのグリッドを比較"""
    try:
        n1, m1, data1 = read_grid_file(file1)
        n2, m2, data2 = read_grid_file(file2)

        if n1 != n2 or m1 != m2:
            print(f"✗ {name1} vs {name2}: グリッドサイズが異なります ({n1}x{m1} vs {n2}x{m2})")
            return False

        max_diff = 0.0
        max_diff_idx = 0
        differences = 0

        for i, (v1, v2) in enumerate(zip(data1, data2)):
            diff = abs(v1 - v2)
            if diff > tolerance:
                differences += 1
            if diff > max_diff:
                max_diff = diff
                max_diff_idx = i

        if differences == 0:
            print(f"✓ {name1} vs {name2}: 完全一致")
            return True
        else:
            row = max_diff_idx // m1
            col = max_diff_idx % m1
            print(f"✗ {name1} vs {name2}: {differences}/{n1*m1} 個の要素が許容誤差を超えています")
            print(f"  最大誤差: {max_diff:.2e} (位置: [{row}][{col}])")
            print(f"  {name1}: {data1[max_diff_idx]:.15f}")
            print(f"  {name2}: {data2[max_diff_idx]:.15f}")
            return False

    except FileNotFoundError as e:
        print(f"✗ {name1} vs {name2}: ファイルが見つかりません - {e}")
        return False
    except Exception as e:
        print(f"✗ {name1} vs {name2}: エラー - {e}")
        return False

# 実装名のマッピング (C名, Rust名, 表示名)
implementations = [
    ("single", "single", "Single Thread"),
    ("unsafe_semaphore", "unsafe_semaphore", "Unsafe Semaphore"),
    ("safe_semaphore", "safe_semaphore", "Safe Semaphore"),
    ("barrier", "barrier", "Barrier"),
    ("openmp", "rayon", "OpenMP/Rayon"),
    ("unsafe_parallel", "unsafe_parallel", "Unsafe Parallel"),
]

print("="*80)
print("実装ごとの結果比較 (C vs Rust)")
print("="*80)
print()

all_match = True
for c_name, rust_name, display_name in implementations:
    c_file = f"c_{c_name}.bin"
    rust_file = f"rust_{rust_name}.bin"

    # Rust専用実装の場合はスキップ
    if c_name == "N/A":
        if os.path.exists(rust_file):
            print(f"○ {display_name}: Rust版のみ実装")
        else:
            print(f"✗ {display_name}: Rust版のファイルが見つかりません ({rust_file})")
            all_match = False
        continue

    if os.path.exists(c_file) and os.path.exists(rust_file):
        match = compare_grids(c_file, rust_file, f"C ({c_name})", f"Rust ({rust_name})", tolerance=1e-10)
        if not match:
            all_match = False
    elif not os.path.exists(c_file):
        print(f"✗ {display_name}: C版のファイルが見つかりません ({c_file})")
        all_match = False
    elif not os.path.exists(rust_file):
        print(f"✗ {display_name}: Rust版のファイルが見つかりません ({rust_file})")
        all_match = False

print()
print("="*80)

if all_match:
    print("\033[0;32m✓ すべての実装で結果が一致しました！\033[0m")
    sys.exit(0)
else:
    print("\033[0;31m✗ 一部の実装で結果が一致しませんでした\033[0m")
    sys.exit(1)

EOF

PYTHON_EXIT=$?

echo ""
echo -e "${CYAN}========================================${NC}"
if [ $PYTHON_EXIT -eq 0 ]; then
    echo -e "${GREEN}テスト完了: すべて一致${NC}"
else
    echo -e "${RED}テスト完了: 不一致あり${NC}"
fi
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "詳細な出力:"
echo -e "  C版: ${BLUE}$TEST_DIR/c_output.txt${NC}"
echo -e "  Rust版: ${BLUE}$TEST_DIR/rust_output.txt${NC}"
echo ""

exit $PYTHON_EXIT
