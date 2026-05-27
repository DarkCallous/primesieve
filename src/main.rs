use primesieve::bitset::*;
use primesieve::linear_siever::*;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::time::Instant;

const CACHE_SIZE: usize = 64 * 1024;

// Prime buckets are indexed by WHEEL residue id:
// 0=>1, 1=>7, 2=>11, 3=>13, 4=>17, 5=>19, 6=>23, 7=>29.
// Each bucket stores only the wheel coordinate k of primes shaped as 30k + WHEEL[id].
type PrimeBuckets = [Vec<usize>; 8];

// Scan only residue pairs that can sum to the current even residue.
// Each tuple is (left residue bucket, right residue bit id, wheel borrow).
// Example: (2, 6, 1) means p comes from bucket 11, E-p has residue 23,
// and the right wheel is k - p_wheel - 1.
macro_rules! scan_pairs {
    ($wheel:expr, $masks:expr, $buckets:expr; $(($left:literal, $right:literal, $carry:literal)),+ $(,)?) => {
        false $(|| scan_bucket::<$carry, $right>(&$buckets[$left], $wheel, $masks))+
    };
}

macro_rules! verify_r {
    // $idx is an index into EVEN_R, so idx 2 verifies numbers shaped as 30k + 4.
    ($idx:literal, $wheel:expr, $masks:expr, $buckets:expr; $($pair:tt),+ $(,)?) => {
        is_prime_after_sub_small::<$idx, 3>($wheel, $masks)
            || is_prime_after_sub_small::<$idx, 5>($wheel, $masks)
            || scan_pairs!($wheel, $masks, $buckets; $($pair),+)
    };
}

fn main() {
    ThreadPoolBuilder::new()
        .num_threads(16)
        .build_global()
        .unwrap();
    let target = 2147483647;
    let start = Instant::now();

    let primes = gen_base_primes(target);
    let stage1 = start.elapsed();
    let base_prime_count = primes.len();

    let target_uwheels = (target as f64 / 240f64).floor() as usize + 1;
    let mask_vec = gen_masks(&primes, 0, target_uwheels);
    let stage2 = start.elapsed();

    let sum = gen_sum(&mask_vec);
    let stage3 = start.elapsed();

    let prime_buckets = gen_prime_buckets(&mask_vec);
    let stage4 = start.elapsed();

    let merged_masks = merge_masks(&mask_vec);
    let verify_goldbach_result = verify_goldbach(target, &merged_masks, &prime_buckets);
    let stage5 = start.elapsed();

    let collected_count = prime_buckets.iter().map(Vec::len).sum::<usize>();

    println!("阶段一耗时: {:?}", stage1);
    println!("找到 {} 个质数", base_prime_count);
    println!("阶段二累计耗时: {:?}", stage2);
    println!("阶段三累计耗时: {:?}", stage3);
    println!("计数器获取质数总数: {}", sum);
    println!("阶段四累计耗时: {:?}", stage4);
    println!("收集器获取集合总数: {}", collected_count);
    println!("阶段五累计耗时: {:?}", stage5);
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

fn merge_masks(masks: &[BitsetU64Wheel30]) -> Vec<u64> {
    let total_len = masks.iter().map(|mask| mask.words().len()).sum();
    let mut result = Vec::with_capacity(total_len);
    for mask in masks {
        result.extend_from_slice(mask.words());
    }
    result
}

fn gen_prime_buckets(masks: &[BitsetU64Wheel30]) -> PrimeBuckets {
    let half = masks.len() / 2;
    let partials: Vec<PrimeBuckets> = masks[..=half]
        .par_iter()
        .map(|mask| {
            let mut local: PrimeBuckets =
                std::array::from_fn(|_| Vec::with_capacity(mask.words().len()));
            for (word_idx, &word) in mask.words().iter().enumerate() {
                let base_wheel = mask.start_wheel() + (word_idx << 3);
                // Bits that remain zero in the sieve are candidate primes; split them by wheel residue.
                let mut unmarked = !word;
                while unmarked != 0 {
                    let bit_idx = unmarked.trailing_zeros() as usize;
                    let id = bit_idx & 0b111;
                    // id is the wheel residue id, so local[id] collects one residue class only.
                    local[id].push(base_wheel + (bit_idx >> 3));
                    unmarked &= unmarked - 1;
                }
            }
            local
        })
        .collect();

    let mut counts = [0usize; 8];
    for local in &partials {
        for id in 0..8 {
            counts[id] += local[id].len();
        }
    }

    let mut result: PrimeBuckets = std::array::from_fn(|id| Vec::with_capacity(counts[id]));
    for local in partials {
        for (id, bucket) in local.into_iter().enumerate() {
            result[id].extend(bucket);
        }
    }
    result
}

fn gen_base_primes(num: usize) -> Vec<(usize, u8)> {
    let base_target = (num as f64).sqrt().ceil() as usize;
    let mut mask = BitsetU64Wheel30::new(base_target / 240 + 1);
    linear_siever(base_target, &mut mask)
}

#[inline(always)]
fn is_prime_pre(num: (usize, u8), masks: &[u64]) -> bool {
    let (wheel, id) = num;
    let pos = wheel >> 3;
    let in_pos = wheel & 0b111;
    let bit_idx = (in_pos << 3) + id as usize;
    let word = unsafe { *masks.get_unchecked(pos) };
    word & (1u64 << bit_idx) == 0
}

#[inline(always)]
fn is_prime_after_sub_small<const EVEN_R_IDX: usize, const SUB: usize>(
    num_wheel: usize,
    masks: &[u64],
) -> bool {
    let even_r = EVEN_R[EVEN_R_IDX];
    let (borrow, right_r) = if even_r >= SUB {
        (0usize, even_r - SUB)
    } else {
        (1usize, even_r + 30 - SUB)
    };
    if num_wheel < borrow {
        return false;
    }
    let right_id = REV_WHEEL[right_r];
    if right_id == 8 {
        return false;
    }
    is_prime_pre((num_wheel - borrow, right_id as u8), masks)
}

#[inline(always)]
fn scan_bucket<const CARRY: usize, const RIGHT_ID: u8>(
    bucket: &[usize],
    num_wheel: usize,
    masks: &[u64],
) -> bool {
    for &prime_wheel in bucket {
        // Buckets are sorted by wheel, so the first overflow ends this residue pair.
        if prime_wheel + CARRY > num_wheel {
            break;
        }
        if is_prime_pre((num_wheel - prime_wheel - CARRY, RIGHT_ID), masks) {
            return true;
        }
    }
    false
}

#[inline(always)]
fn verify_wheel(num_wheel: usize, masks: &[u64], buckets: &PrimeBuckets) -> bool {
    // One full wheel contains the 15 even residues 0, 2, ..., 28.
    verify_r!(0, num_wheel, masks, buckets; (0, 7, 1), (1, 6, 1), (2, 5, 1), (3, 4, 1), (4, 3, 1), (5, 2, 1), (6, 1, 1), (7, 0, 1))
        && verify_r!(1, num_wheel, masks, buckets; (0, 0, 0), (3, 5, 1), (5, 3, 1))
        && verify_r!(2, num_wheel, masks, buckets; (2, 6, 1), (4, 4, 1), (6, 2, 1))
        && verify_r!(3, num_wheel, masks, buckets; (1, 7, 1), (3, 6, 1), (4, 5, 1), (5, 4, 1), (6, 3, 1), (7, 1, 1))
        && verify_r!(4, num_wheel, masks, buckets; (0, 1, 0), (1, 0, 0), (5, 5, 1))
        && verify_r!(5, num_wheel, masks, buckets; (2, 7, 1), (4, 6, 1), (6, 4, 1), (7, 2, 1))
        && verify_r!(6, num_wheel, masks, buckets; (0, 2, 0), (2, 0, 0), (3, 7, 1), (5, 6, 1), (6, 5, 1), (7, 3, 1))
        && verify_r!(7, num_wheel, masks, buckets; (0, 3, 0), (1, 1, 0), (3, 0, 0))
        && verify_r!(8, num_wheel, masks, buckets; (4, 7, 1), (6, 6, 1), (7, 4, 1))
        && verify_r!(9, num_wheel, masks, buckets; (0, 4, 0), (1, 2, 0), (2, 1, 0), (4, 0, 0), (5, 7, 1), (7, 5, 1))
        && verify_r!(10, num_wheel, masks, buckets; (0, 5, 0), (1, 3, 0), (3, 1, 0), (5, 0, 0))
        && verify_r!(11, num_wheel, masks, buckets; (2, 2, 0), (6, 7, 1), (7, 6, 1))
        && verify_r!(12, num_wheel, masks, buckets; (0, 6, 0), (1, 4, 0), (2, 3, 0), (3, 2, 0), (4, 1, 0), (6, 0, 0))
        && verify_r!(13, num_wheel, masks, buckets; (1, 5, 0), (3, 3, 0), (5, 1, 0))
        && verify_r!(14, num_wheel, masks, buckets; (2, 4, 0), (4, 2, 0), (7, 7, 1))
}

#[inline(always)]
fn verify_wheel0(masks: &[u64], buckets: &PrimeBuckets) -> bool {
    // The global verification starts at 8, so wheel 0 skips residues 0, 2, 4, and 6.
    verify_r!(4, 0, masks, buckets; (0, 1, 0), (1, 0, 0), (5, 5, 1))
        && verify_r!(5, 0, masks, buckets; (2, 7, 1), (4, 6, 1), (6, 4, 1), (7, 2, 1))
        && verify_r!(6, 0, masks, buckets; (0, 2, 0), (2, 0, 0), (3, 7, 1), (5, 6, 1), (6, 5, 1), (7, 3, 1))
        && verify_r!(7, 0, masks, buckets; (0, 3, 0), (1, 1, 0), (3, 0, 0))
        && verify_r!(8, 0, masks, buckets; (4, 7, 1), (6, 6, 1), (7, 4, 1))
        && verify_r!(9, 0, masks, buckets; (0, 4, 0), (1, 2, 0), (2, 1, 0), (4, 0, 0), (5, 7, 1), (7, 5, 1))
        && verify_r!(10, 0, masks, buckets; (0, 5, 0), (1, 3, 0), (3, 1, 0), (5, 0, 0))
        && verify_r!(11, 0, masks, buckets; (2, 2, 0), (6, 7, 1), (7, 6, 1))
        && verify_r!(12, 0, masks, buckets; (0, 6, 0), (1, 4, 0), (2, 3, 0), (3, 2, 0), (4, 1, 0), (6, 0, 0))
        && verify_r!(13, 0, masks, buckets; (1, 5, 0), (3, 3, 0), (5, 1, 0))
        && verify_r!(14, 0, masks, buckets; (2, 4, 0), (4, 2, 0), (7, 7, 1))
}

fn verify_goldbach(num: usize, masks: &[u64], buckets: &PrimeBuckets) -> bool {
    if num < 8 {
        return true;
    }

    let max_wheel = num / 30;
    verify_wheel0(masks, buckets)
        && (1..=max_wheel)
            .into_par_iter()
            .all(|wheel| verify_wheel(wheel, masks, buckets))
}
