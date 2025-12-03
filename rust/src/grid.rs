pub const N: usize = 128;  // x方向セル数
pub const M: usize = 128;  // y方向セル数
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