pub use crate::wheel_tables::*;

pub struct BitsetU64Wheel30 {
    start: usize,
    end: usize,
    mask: Vec<u64>,
}

impl BitsetU64Wheel30 {
    pub fn new(uwheel: usize) -> BitsetU64Wheel30 {
        BitsetU64Wheel30 {
            start: 0,
            end: uwheel,
            mask: vec![0u64; uwheel],
        }
    }

    pub fn piece(start_uwheel: usize, end_uwheel: usize) -> BitsetU64Wheel30 {
        BitsetU64Wheel30 {
            start: start_uwheel,
            end: end_uwheel,
            mask: vec![0u64; end_uwheel - start_uwheel],
        }
    }

    pub fn is_marked(&self, wheel: usize, id: u8) -> bool {
        let pos = wheel >> 3;
        let in_pos = wheel & 0b111;
        let word = unsafe { *self.mask.get_unchecked(pos - self.start) };
        let bit_idx = (in_pos << 3) + id as usize;
        word & (1 << bit_idx) != 0
    }

    pub fn mark(&mut self, wheel: usize, id: u8) {
        let pos = wheel >> 3;
        let in_pos = wheel & 0b111;
        let bit_idx = (in_pos << 3) + id as usize;
        unsafe { *self.mask.get_unchecked_mut(pos - self.start) |= 1 << bit_idx };
    }

    pub fn mul((wheel_a, id_a): (usize, u8), (wheel_b, id_b): (usize, u8)) -> (usize, u8) {
        let (add_wheel, new_id) = MUL30_TABLE[id_a as usize][id_b as usize];
        let new_wheel = 30 * wheel_a * wheel_b
            + WHEEL[id_a as usize] * wheel_b
            + WHEEL[id_b as usize] * wheel_a
            + add_wheel as usize;
        (new_wheel, new_id)
    }

    pub fn start_wheel(&self) -> usize {
        self.start << 3
    }

    pub fn end_wheel(&self) -> usize {
        (self.end << 3) - 1
    }

    pub fn prime_counts(&self) -> usize {
        self.mask.iter().map(|i| i.count_zeros() as usize).sum()
    }

    pub fn collect_primes(&self) -> Vec<(usize, u8)> {
        let estimated_capacity = self.prime_counts();
        let mut result = Vec::with_capacity(estimated_capacity);

        for (u64_idx, &word) in self.mask.iter().enumerate() {
            let base_wheel = (self.start + u64_idx) << 3;

            // 反转 word，因为我们要找 0 位（未标记的质数）
            let mut unmarked = !word;

            while unmarked != 0 {
                // 找到最低位的 1（对应原始 word 中的 0）
                let bit_idx = unmarked.trailing_zeros() as usize;

                // 计算对应的 wheel 和 id
                let wheel_in_u64 = bit_idx >> 3;
                let id = (bit_idx & 0b111) as u8;
                let wheel = base_wheel + wheel_in_u64;
                result.push((wheel, id));
                // 清除这一位，继续扫描
                unmarked &= unmarked - 1;
            }
        }

        result
    }
}
