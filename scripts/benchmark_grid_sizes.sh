#!/bin/bash

# グリッドサイズ別ベンチマークスクリプト
# Rust vs Unsafe Rust、Rust vs C の比較を行う

set -e

# プロジェクトルート（scriptsディレクトリの親ディレクトリ）
SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"

# 測定するグリッドサイズ
GRID_SIZES=(512 1024 2048 4096)

# 結果を保存するディレクトリ
RESULTS_DIR="$SCRIPTS_DIR/benchmark_results/grid_size_comparison"
mkdir -p "$RESULTS_DIR"

echo "=== グリッドサイズ別ベンチマーク ==="
echo "測定するグリッドサイズ: ${GRID_SIZES[@]}"
echo ""

# 結果ファイルの初期化
RUST_RESULTS="$RESULTS_DIR/rust_results.txt"
C_RESULTS="$RESULTS_DIR/c_results.txt"

echo "GridSize,Implementation,Min,Median,Avg,Max" > "$RUST_RESULTS"
echo "GridSize,Implementation,Min,Median,Avg,Max" > "$C_RESULTS"

# 各グリッドサイズで測定
for SIZE in "${GRID_SIZES[@]}"; do
    echo "========================================="
    echo "グリッドサイズ: ${SIZE}x${SIZE}"
    echo "========================================="

    # Rustのgrid.rsを更新
    cat > "$PROJECT_ROOT/rust/src/grid.rs" << EOF
pub const N: usize = ${SIZE};  // x方向セル数
pub const M: usize = ${SIZE};  // y方向セル数
pub const TIME_STEPS: usize = 100;  //ステップ数
pub const WARMUP_STEPS: usize = 10; //ウォームアップ数
pub const DT: f64 = 0.1;  //時間刻み幅
pub const DX: f64 = 1.0;  //グリッドの1セルの「物理的距離」
pub const ALPHA: f64 = 0.8;  // 拡散係数


#[derive(Clone,Debug)]
pub struct Grid {
    pub data: Vec<f64>,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            data: vec![0.0; N * M],
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        let mut grid = Grid::default();
        // 格子の中心に熱源を設定
        grid.data[N / 2 * M + M / 2] = 100.0;
        grid
    }

    // 格子の温度を表示
    pub fn print(&self) {
        for i in 0..N {
            for j in 0..M {
                print!("{:6.2} ", self.data[i * M + j]);
            }
            println!();
        }
    }

    // バイナリ形式でファイルに保存
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;

        // ヘッダー: N, M (4バイトずつ)
        file.write_all(&(N as u32).to_le_bytes())?;
        file.write_all(&(M as u32).to_le_bytes())?;

        // データ: f64配列
        for &val in &self.data {
            file.write_all(&val.to_le_bytes())?;
        }
        Ok(())
    }

    // バイナリファイルから読み込み
    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;

        // ヘッダー読み込み
        let mut buf_n = [0u8; 4];
        let mut buf_m = [0u8; 4];
        file.read_exact(&mut buf_n)?;
        file.read_exact(&mut buf_m)?;

        let n = u32::from_le_bytes(buf_n) as usize;
        let m = u32::from_le_bytes(buf_m) as usize;

        if n != N || m != M {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Grid size mismatch: expected {}x{}, got {}x{}", N, M, n, m)
            ));
        }

        // データ読み込み
        let mut data = Vec::with_capacity(N * M);
        let mut buf = [0u8; 8];
        for _ in 0..N * M {
            file.read_exact(&mut buf)?;
            data.push(f64::from_le_bytes(buf));
        }

        Ok(Grid { data })
    }
}
EOF

    # C言語のjacobi_common.hを更新
    sed -i.bak "s/#define N [0-9]*/#define N ${SIZE}/" "$PROJECT_ROOT/c/common/jacobi_common.h"
    sed -i.bak "s/#define M [0-9]*/#define M ${SIZE}/" "$PROJECT_ROOT/c/common/jacobi_common.h"
    sed -i.bak "s/#define TIME_STEPS [0-9]*/#define TIME_STEPS 100/" "$PROJECT_ROOT/c/common/jacobi_common.h"

    echo ""
    echo "--- Rustベンチマーク実行 ---"
    cd "$PROJECT_ROOT/rust"
    cargo build --release 2>&1 | grep -v "Compiling\|Finished" || true
    OUTPUT=$(cargo run --release 2>&1)
    echo "$OUTPUT"
    cd "$SCRIPTS_DIR"

    # Rustの結果を抽出してCSVに保存
    echo "$OUTPUT" | grep -A 5 "Single Thread:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",SingleThread,"$2}'  >> "$RUST_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Safe Semaphore:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",SafeSemaphore,"$2}' >> "$RUST_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Barrier:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",Barrier,"$2}' >> "$RUST_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Barrier Unsafe:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",BarrierUnsafe,"$2}' >> "$RUST_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Rayon:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",Rayon,"$2}' >> "$RUST_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Rayon Unsafe:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",RayonUnsafe,"$2}' >> "$RUST_RESULTS" || true

    echo ""
    echo "--- Cベンチマーク実行 ---"
    cd "$PROJECT_ROOT/c"
    make clean > /dev/null 2>&1
    make 2>&1 | grep -v "gcc\|clang" || true
    OUTPUT=$(./jacobi_bench 2>&1)
    echo "$OUTPUT"
    cd "$SCRIPTS_DIR"

    # Cの結果を抽出してCSVに保存
    echo "$OUTPUT" | grep -A 5 "Single Thread:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",SingleThread,"$2}' >> "$C_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Safe Semaphore:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",SafeSemaphore,"$2}' >> "$C_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "Barrier:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",Barrier,"$2}' >> "$C_RESULTS" || true
    echo "$OUTPUT" | grep -A 5 "OpenMP:" | grep "平均値:" | awk -v size="${SIZE}" '{gsub(/ms/,""); print size",OpenMP,"$2}' >> "$C_RESULTS" || true

    echo ""
done

echo "========================================="
echo "すべての測定が完了しました"
echo "結果は以下に保存されています:"
echo "  Rust: $RUST_RESULTS"
echo "  C:    $C_RESULTS"
echo "========================================="

# 元の設定に戻す (2048x2048, TIME_STEPS=1000)
cat > rust/src/grid.rs << 'EOF'
pub const N: usize = 2048;  // x方向セル数
pub const M: usize = 2048;  // y方向セル数
pub const TIME_STEPS: usize = 1000;  //ステップ数
pub const WARMUP_STEPS: usize = 10; //ウォームアップ数
pub const DT: f64 = 0.1;  //時間刻み幅
pub const DX: f64 = 1.0;  //グリッドの1セルの「物理的距離」
pub const ALPHA: f64 = 0.8;  // 拡散係数


#[derive(Clone,Debug)]
pub struct Grid {
    pub data: Vec<f64>,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            data: vec![0.0; N * M],
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        let mut grid = Grid::default();
        // 格子の中心に熱源を設定
        grid.data[N / 2 * M + M / 2] = 100.0;
        grid
    }

    // 格子の温度を表示
    pub fn print(&self) {
        for i in 0..N {
            for j in 0..M {
                print!("{:6.2} ", self.data[i * M + j]);
            }
            println!();
        }
    }

    // バイナリ形式でファイルに保存
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;

        // ヘッダー: N, M (4バイトずつ)
        file.write_all(&(N as u32).to_le_bytes())?;
        file.write_all(&(M as u32).to_le_bytes())?;

        // データ: f64配列
        for &val in &self.data {
            file.write_all(&val.to_le_bytes())?;
        }
        Ok(())
    }

    // バイナリファイルから読み込み
    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;

        // ヘッダー読み込み
        let mut buf_n = [0u8; 4];
        let mut buf_m = [0u8; 4];
        file.read_exact(&mut buf_n)?;
        file.read_exact(&mut buf_m)?;

        let n = u32::from_le_bytes(buf_n) as usize;
        let m = u32::from_le_bytes(buf_m) as usize;

        if n != N || m != M {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Grid size mismatch: expected {}x{}, got {}x{}", N, M, n, m)
            ));
        }

        // データ読み込み
        let mut data = Vec::with_capacity(N * M);
        let mut buf = [0u8; 8];
        for _ in 0..N * M {
            file.read_exact(&mut buf)?;
            data.push(f64::from_le_bytes(buf));
        }

        Ok(Grid { data })
    }
}
EOF

sed -i.bak "s/#define N [0-9]*/#define N 2048/" c/common/jacobi_common.h
sed -i.bak "s/#define M [0-9]*/#define M 2048/" c/common/jacobi_common.h
sed -i.bak "s/#define TIME_STEPS [0-9]*/#define TIME_STEPS 1000/" c/common/jacobi_common.h

echo "設定を元に戻しました (2048x2048, TIME_STEPS=1000)"
