pub const N: usize = 1000;  // x方向セル数
pub const M: usize = 1000;  // y方向セル数
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
}
