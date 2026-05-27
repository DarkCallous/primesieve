pub const WHEEL: [usize; 8] = [1, 7, 11, 13, 17, 19, 23, 29];
pub const REV_WHEEL: [usize; 30] = [
    8, // 0
    0, // 1 -> WHEEL[0]
    8, // 2
    8, // 3
    8, // 4
    8, // 5
    8, // 6
    1, // 7 -> WHEEL[1]
    8, // 8
    8, // 9
    8, //10
    2, //11 -> WHEEL[2]
    8, //12
    3, //13 -> WHEEL[3]
    8, //14
    8, //15
    8, //16
    4, //17 -> WHEEL[4]
    8, //18
    5, //19 -> WHEEL[5]
    8, //20
    8, //21
    8, //22
    6, //23 -> WHEEL[6]
    8, //24
    8, //25
    8, //26
    8, //27
    8, //28
    7, //29 -> WHEEL[7]
];

const fn remainder_to_id(rem: u8) -> u8 {
    match rem {
        1 => 0,
        7 => 1,
        11 => 2,
        13 => 3,
        17 => 4,
        19 => 5,
        23 => 6,
        29 => 7,
        _ => 255, // 不应该出现
    }
}

const fn build_mul30_table() -> [[(u8, u8); 8]; 8] {
    let mut table = [[(0u8, 0u8); 8]; 8];
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

pub const MUL30_TABLE: [[(u8, u8); 8]; 8] = build_mul30_table();

pub const EVEN_R: [usize; 15] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28];

pub type WheelPair = (u8, u8, u8); // (i, j, carry)

pub const fn make_wheel_pairs_even() -> [[WheelPair; 16]; 15] {
    let mut table = [[(8u8, 8u8, 0u8); 16]; 15];
    let mut ri = 0;
    while ri < 15 {
        let r = EVEN_R[ri];
        let mut count = 0;
        let mut i = 0;
        while i < 8 {
            let rp = WHEEL[i];
            let mut j = 0;
            while j < 8 {
                let rq = WHEEL[j];
                let sum = rp + rq;
                if sum % 30 == r {
                    let carry = if sum >= 30 { 1 } else { 0 };
                    table[ri][count] = (i as u8, j as u8, carry);
                    count += 1;
                }
                j += 1;
            }
            i += 1;
        }
        ri += 1;
    }
    table
}

pub const WHEEL_PAIRS_EVEN: [[WheelPair; 16]; 15] = make_wheel_pairs_even();
