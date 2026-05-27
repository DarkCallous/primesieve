use primesieve::bitset::*;
use primesieve::linear_siever::*;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::time::Instant;

const CACHE_SIZE: usize = 64 * 1024;

fn main() {
    ThreadPoolBuilder::new()
        .num_threads(16)
        .build_global()
        .unwrap();
    let target = 2147483647;
    let start = Instant::now();
    let primes = gen_base_primes(target);
    let duration = start.elapsed();
    println!("阶段一耗时: {:?}", duration);
    println!("找到 {} 个质数", primes.len());
    let target_uwheels = (target as f64 / 240f64).floor() as usize + 1;
    let mask_vec = gen_masks(&primes, 0, target_uwheels);
    let duration = start.elapsed();
    println!("阶段二累计耗时: {:?}", duration);
    let sum = gen_sum(&mask_vec);
    let duration = start.elapsed();
    println!("阶段三累计耗时: {:?}", duration);
    println!("计数器获取质数总数: {}", sum);
    let primes_vec: Vec<Vec<(usize, u8)>> = gen_prime_collection(&mask_vec);
    let half = primes_vec.len() / 2;
    let mut primes_vec_collected = Vec::with_capacity(sum);
    for v in &primes_vec[..=half] {
        primes_vec_collected.extend_from_slice(v);
    }
    let duration = start.elapsed();
    println!("阶段四累计耗时: {:?}", duration);
    println!("收集器获取集合总数: {}", primes_vec_collected.len());
    let verify_goldbach_result = verify_goldbach(target, &mask_vec, &primes_vec_collected);
    let duration = start.elapsed();
    println!("阶段五累计耗时: {:?}", duration);
    println!("验证结果: {}", verify_goldbach_result);
}

fn gen_windows(start: usize, end: usize, step: usize) -> impl Iterator<Item = (usize, usize)> {
    (start..end).step_by(step).map(move |val| {
        let end = (val + step).min(end);
        (val, end)
    })
}

fn gen_masks(
    primes: &[(usize, u8)],
    start_uwheel: usize,
    end_uwheel: usize,
) -> Vec<BitsetU64Wheel30> {
    let iters: Vec<_> = gen_windows(start_uwheel, end_uwheel, CACHE_SIZE).collect();
    iters
        .par_iter()
        .map(|&(start, end)| linear_siever_marker(primes, start, end))
        .collect()
}

fn gen_sum(masks: &[BitsetU64Wheel30]) -> usize {
    masks.par_iter().map(|mask| mask.prime_counts()).sum()
}

fn gen_prime_collection(masks: &[BitsetU64Wheel30]) -> Vec<Vec<(usize, u8)>> {
    masks.par_iter().map(|mask| mask.collect_primes()).collect()
}

fn gen_base_primes(num: usize) -> Vec<(usize, u8)> {
    let base_target = (num as f64).sqrt().ceil() as usize;
    let mut mask = BitsetU64Wheel30::new(base_target / 240 + 1);
    linear_siever(base_target, &mut mask)
}

fn is_prime(num: usize, masks: &[BitsetU64Wheel30]) -> bool {
    //不需要判3/5，因为L + R(L < R)，所以R不可能是3/5，而该函数仅用于R
    let wheel = num / 30;
    let modx = num - wheel * 30;
    let id = REV_WHEEL[modx];
    if id == 8 {
        return false;
    }
    let storage = (wheel >> 3) / CACHE_SIZE;
    return !masks[storage].is_marked(wheel, id as u8);
}

fn verify_goldbach_single(num: usize, masks: &[BitsetU64Wheel30], primes: &[(usize, u8)]) -> bool {
    if is_prime(num - 3, masks) {
        return true;
    }
    if is_prime(num - 5, masks) {
        return true;
    }
    primes
        .iter()
        .map(|&(prime_wheel, prime_id)| prime_wheel * 30 + prime_id as usize)
        .map(|prime| num - prime)
        .any(|diff| is_prime(diff, masks))
}

fn is_prime_pre(num: (usize, u8), masks: &[BitsetU64Wheel30]) -> bool {
    let (wheel, id) = num;
    let storage = (wheel >> 3) / CACHE_SIZE;
    return !masks[storage].is_marked(wheel, id as u8);
}

fn verify_goldbach_single_pre(
    num: (usize, u8),
    masks: &[BitsetU64Wheel30],
    primes: &[(usize, u8)],
) -> bool {
    true
}

fn verify_goldbach(num: usize, masks: &[BitsetU64Wheel30], primes: &[(usize, u8)]) -> bool {
    let max_index = (num - 8) / 2 + 1;
    (0..max_index)
        .into_par_iter()
        .map(|i| 8 + i * 2)
        .all(|x| verify_goldbach_single(x, masks, primes))
}
