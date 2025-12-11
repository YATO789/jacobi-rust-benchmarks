// 境界チェックの有無を確認するテストコード

fn safe_access(data: &Vec<f64>, i: usize, m: usize) -> f64 {
    let idx = i * m + 5;
    data[idx + 1] + data[idx - 1] + data[idx + m] + data[idx - m] - 4.0 * data[idx]
}

fn unsafe_access(data: &Vec<f64>, i: usize, m: usize) -> f64 {
    unsafe {
        let ptr = data.as_ptr();
        let idx = i * m + 5;
        let curr = ptr.add(idx);

        *curr.add(1) + *curr.sub(1) + *curr.add(m) + *curr.sub(m) - 4.0 * *curr
    }
}

fn main() {
    let data = vec![1.0; 1024 * 1024];

    // ウォームアップ
    for i in 1..1000 {
        let _ = safe_access(&data, i, 1024);
        let _ = unsafe_access(&data, i, 1024);
    }

    // ベンチマーク
    let start = std::time::Instant::now();
    let mut sum = 0.0;
    for _ in 0..1000000 {
        sum += safe_access(&data, 512, 1024);
    }
    let safe_time = start.elapsed();
    println!("Safe version: {:?}, sum={}", safe_time, sum);

    let start = std::time::Instant::now();
    let mut sum = 0.0;
    for _ in 0..1000000 {
        sum += unsafe_access(&data, 512, 1024);
    }
    let unsafe_time = start.elapsed();
    println!("Unsafe version: {:?}, sum={}", unsafe_time, sum);

    println!("Speedup: {:.2}x", safe_time.as_secs_f64() / unsafe_time.as_secs_f64());
}
