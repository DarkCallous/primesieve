use crate::bitset::*;

pub fn linear_siever(target: usize, mask: &mut BitsetU64Wheel30) -> Vec<(usize, u8)> {
    let size = if target > 100 {
        (target as f64 / (target as f64).ln() * 1.1).ceil() as usize
    } else {
        30
    };
    let mut result = Vec::with_capacity(size);
    let max_round = target / 30 + 1;
    let no_more_need_round = max_round / 7 + 1;
    let iter = (0..=max_round)
        .flat_map(|w| (0..=7u8).map(move |id| (w, id)))
        .skip(1);
    for (scan_wheel, scan_id) in iter {
        let is_prime = !mask.is_marked(scan_wheel, scan_id);
        if is_prime {
            result.push((scan_wheel, scan_id));
        }
        if scan_wheel > no_more_need_round {
            continue;
        }
        for &(old_wheel, old_id) in result.iter() {
            let (new_wheel, new_id) =
                BitsetU64Wheel30::mul((scan_wheel, scan_id), (old_wheel, old_id));
            if new_wheel > max_round {
                break;
            }
            mask.mark(new_wheel, new_id);
        }
    }
    result
}

pub fn linear_siever_marker(
    base_primes: &[(usize, u8)],
    start_uwheel: usize,
    end_uwheel: usize,
) -> BitsetU64Wheel30 {
    let mut result = BitsetU64Wheel30::piece(start_uwheel, end_uwheel);
    for prime in base_primes.iter() {
        linear_siever_marker_single(prime, &mut result);
    }
    result
}

pub fn find_ge_wheel_coord(val: usize) -> (usize, u8) {
    let wheel = val / 30;
    let modx = val % 30;
    let mut id = WHEEL.iter().position(|val| val >= &modx).unwrap_or(8);
    if wheel == 0 {
        id = id.max(1);
    }
    (wheel, id as u8)
}

pub fn linear_siever_marker_single(prime: &(usize, u8), mask: &mut BitsetU64Wheel30) {
    let &(prime_wheel, prime_id) = prime;
    let start_num = ((mask.start_wheel() * 30) as f64
        / (prime_wheel * 30 + WHEEL[prime_id as usize]) as f64)
        .ceil() as usize;
    let end_num = ((mask.end_wheel() * 30 + 30) as f64
        / (prime_wheel * 30 + WHEEL[prime_id as usize]) as f64)
        .ceil() as usize;
    let (start_wheel, start_id) = find_ge_wheel_coord(start_num);
    let (end_wheel, end_id) = find_ge_wheel_coord(end_num);
    if start_wheel < end_wheel {
        for id in start_id..=7 {
            let to_mark = BitsetU64Wheel30::mul((start_wheel, id), (prime_wheel, prime_id));
            mask.mark(to_mark.0, to_mark.1);
        }
        for wheel in start_wheel + 1..end_wheel {
            for id in 0u8..=7 {
                let to_mark = BitsetU64Wheel30::mul((wheel, id), (prime_wheel, prime_id));
                mask.mark(to_mark.0, to_mark.1);
            }
        }
        for id in 0u8..end_id {
            let to_mark = BitsetU64Wheel30::mul((end_wheel, id), (prime_wheel, prime_id));
            mask.mark(to_mark.0, to_mark.1);
        }
    } else {
        for id in start_id..end_id {
            let to_mark = BitsetU64Wheel30::mul((start_wheel, id), (prime_wheel, prime_id));
            mask.mark(to_mark.0, to_mark.1);
        }
    }
}
