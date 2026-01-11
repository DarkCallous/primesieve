pub const WHEEL: [usize; 8] = [1,7,11,13,17,19,23,29]; 
pub const REV_WHEEL: [usize; 30] = [
    8,  // 0
    0,  // 1 -> WHEEL[0]
    8,  // 2
    8,  // 3
    8,  // 4
    8,  // 5
    8,  // 6
    1,  // 7 -> WHEEL[1]
    8,  // 8
    8,  // 9
    8,  //10
    2,  //11 -> WHEEL[2]
    8,  //12
    3,  //13 -> WHEEL[3]
    8,  //14
    8,  //15
    8,  //16
    4,  //17 -> WHEEL[4]
    8,  //18
    5,  //19 -> WHEEL[5]
    8,  //20
    8,  //21
    8,  //22
    6,  //23 -> WHEEL[6]
    8,  //24
    8,  //25
    8,  //26
    8,  //27
    8,  //28
    7,  //29 -> WHEEL[7]
];

const fn remainder_to_id(rem: u8) -> u8 {
    match rem {
        1 => 0, 7 => 1, 11 => 2, 13 => 3,
        17 => 4, 19 => 5, 23 => 6, 29 => 7,
        _ => 255 // 不应该出现
    }
}

const fn build_mul30_table() -> [[(u8,u8);8];8] {
    let mut table = [[(0u8,0u8);8];8];
    let mut i = 0;
    while i < 8 {
        let mut j = 0;
        while j < 8 {
            let prod = WHEEL[i] as u16 * WHEEL[j] as u16;
            table[i][j] = ((prod / 30) as u8, remainder_to_id((prod % 30) as u8));
            j += 1;
        }
        i += 1;
    }
    table
}

pub const MUL30_TABLE: [[(u8,u8);8];8] = build_mul30_table();

pub struct BitsetU64Wheel30{
    start: usize,
    end: usize,
    mask: Vec<u64>,
}

impl BitsetU64Wheel30{
    pub fn new(uwheel: usize) -> BitsetU64Wheel30{
        BitsetU64Wheel30{
            start: 0,
            end: uwheel,
            mask: vec![0u64; uwheel]
        }
    }
    
    pub fn piece(start_uwheel: usize, end_uwheel: usize) -> BitsetU64Wheel30{
        BitsetU64Wheel30 { start: start_uwheel, end: end_uwheel, mask: vec![0u64; end_uwheel - start_uwheel] }
    }

    pub fn is_marked(&self, wheel: usize, id: u8) -> bool{
        let pos = wheel >> 3;
        let in_pos = wheel & 0b111;
        let word = unsafe{*self.mask.get_unchecked(pos - self.start)};
        let bit_idx = (in_pos << 3) + id as usize;
        word & (1<<bit_idx) != 0
    }

    pub fn mark(&mut self, wheel: usize, id: u8){
        let pos = wheel >> 3;
        let in_pos = wheel & 0b111;
        let bit_idx = (in_pos << 3) + id as usize;
        unsafe{*self.mask.get_unchecked_mut(pos - self.start) |= 1<<bit_idx};
    }

    pub fn mul((wheel_a, id_a): (usize, u8), (wheel_b, id_b): (usize, u8)) -> (usize, u8){
        let (add_wheel, new_id) = MUL30_TABLE[id_a as usize][id_b as usize];
        let new_wheel = 
            30 * wheel_a * wheel_b 
            + WHEEL[id_a as usize] * wheel_b 
            + WHEEL[id_b as usize] * wheel_a 
            + add_wheel as usize;
        (new_wheel, new_id)
    }

    pub fn start_wheel(&self) -> usize{
        self.start << 3
    }

    pub fn end_wheel(&self) -> usize{
        (self.end << 3) - 1 
    }

    pub fn prime_counts(&self) -> usize{
        self.mask.iter().map(|i| i.count_zeros() as usize).sum()
    }

    pub fn collect_primes(&self) -> Vec<(usize, u8)>{
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