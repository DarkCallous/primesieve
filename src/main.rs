use primesieve::linear_siever::*;
use primesieve::bitset::*;
use std::time::Instant;

fn main() {
    let target = 2147483647;
    let mut mask = BitsetU64Wheel30::new(target / 30 + 1);
    
    let start = Instant::now();
    let primes = linear_siever_marker(target, &mut mask);
    let duration = start.elapsed();
    
    //println!("找到 {} 个质数", primes.len());
    println!("耗时: {:?}", duration);
    println!("耗时: {:.2} ms", duration.as_secs_f64() * 1000.0);
}
