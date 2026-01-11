use crate::bitset::*;

pub fn linear_siever(target: usize, mask: &mut BitsetU64Wheel30) -> Vec<(usize, u8)>{
    let size = if target > 100{ (target as f64 / (target as f64).ln() * 1.1).ceil() as usize } else{30};
    let mut result = Vec::with_capacity(size);
    let max_round = target / 30 + 1;
    let no_more_need_round = max_round / 7 + 1;
    let iter = (0..=max_round).flat_map(|w| (0..=7u8).map(move |id| (w, id))).skip(1);
    for (scan_wheel, scan_id) in iter{
            let is_prime = !mask.is_marked(scan_wheel, scan_id);
            if is_prime{
                result.push((scan_wheel, scan_id));
            }
            if scan_wheel > no_more_need_round{continue;} 
            for &(old_wheel, old_id) in result.iter(){
                let (new_wheel, new_id) = BitsetU64Wheel30::mul((scan_wheel, scan_id), (old_wheel, old_id));
                if new_wheel > max_round {break;}
                mask.mark(new_wheel, new_id);
                //let num = 30 * scan_wheel + WHEEL[scan_id as usize];
                //let old_val = 30 * old_wheel + WHEEL[old_id as usize];
                //if num % old_val == 0 { break; }
            }
        }
    result
}

pub fn linear_siever_marker(target: usize, mask: &mut BitsetU64Wheel30){
    let part1 = (target as f64).sqrt().ceil() as usize;
    let max_round = target / 30 + 1;
    let no_more_need_round = max_round / 7 + 1;
    let part1_primes = linear_siever(part1, mask);
    let iter = (0..=no_more_need_round).flat_map(|w| (0..=7u8).map(move |id| (w, id)));
    for (scan_wheel, scan_id) in iter{
        for &(old_wheel, old_id) in part1_primes.iter(){
            let (new_wheel, new_id) = BitsetU64Wheel30::mul((scan_wheel, scan_id), (old_wheel, old_id));
            if new_wheel > max_round {break;}
            mask.mark(new_wheel, new_id);
        }
    }
}