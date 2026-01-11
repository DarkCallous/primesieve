pub const WHEEL: [usize; 8] = [1,7,11,13,17,19,23,29]; 
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
    mask: Vec<u64>,
}

impl BitsetU64Wheel30{
    pub fn new(wheel: usize) -> BitsetU64Wheel30{
        BitsetU64Wheel30{
            mask: vec![0u64; (wheel >> 3) + 1]
        }
    }

    pub fn is_marked(&self, wheel: usize, id: u8) -> bool{
        let pos = wheel >> 3;
        let in_pos = wheel & 0b111;
        let word = unsafe{*self.mask.get_unchecked(pos)};
        let bit_idx = (in_pos << 3) + id as usize;
        word & (1<<bit_idx) != 0
    }

    pub fn mark(&mut self, wheel: usize, id: u8){
        let pos = wheel >> 3;
        let in_pos = wheel & 0b111;
        let bit_idx = (in_pos << 3) + id as usize;
        unsafe{*self.mask.get_unchecked_mut(pos) |= 1<<bit_idx};
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
}