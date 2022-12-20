// macro for calculating tables
macro_rules! init {
    ($init:stmt, $idx:expr, $initial:expr, $func:expr) => {{
        let mut res = [$initial; 64];
        $init
        while $idx < 64 {
            res[$idx] = $func;
            $idx += 1;
        }
        res
    }};
}

// pcs / sides
pub const E: usize = 0;
pub const WH: usize = 0;
pub const BL: usize = 1;
pub const P: usize = 2;
pub const N: usize = 3;
pub const B: usize = 4;
pub const R: usize = 5;
pub const Q: usize = 6;
pub const K: usize = 7;

// move flags
pub const QUIET: u8 = 0;
pub const DBL: u8 = 1;
pub const KS: u8 = 2;
pub const QS: u8 = 3;
pub const CAP: u8 = 4;
pub const ENP: u8 = 5;
pub const PROMO: u8 = 8;
pub const BPROMO: u8 = 9;
pub const RPROMO: u8 = 10;
pub const QPROMO: u8 = 11;
pub const PROMO_CAP: u8 = 12;
pub const BPROMO_CAP: u8 = 13;
pub const RPROMO_CAP: u8 = 14;
pub const QPROMO_CAP: u8 = 15;

// castling
pub const WQS: u8 = 8;
pub const WKS: u8 = 4;
pub const BQS: u8 = 2;
pub const BKS: u8 = 1;
pub const SIDES: [u8; 2] = [WKS | WQS, BKS | BQS];
pub const CKM: [u64; 2] = [160, 0xA000000000000000];
pub const CQM: [u64; 2] = [9, 0x0900000000000000];
pub const B1C1D1: u64 = 14;
pub const F1G1: u64 = 96;
pub const B8C8D8: u64 = 0x0E00000000000000;
pub const F8G8: u64 = 0x6000000000000000;
pub const CR: [u8; 64] = init!(let mut idx = 0, idx, 0, match idx {0 => 7, 4 => 3, 7 => 11, 56 => 13, 60 => 12, 63 => 14, _ => 15,});

// attacks
pub const MSB: u64 = 0x80_00_00_00_00_00_00_00;
pub const LSB: u64 = 1;

// for promotions / double pushes
pub const PENRANK: [u64; 2] = [0x00FF000000000000, 0x000000000000FF00];
pub const DBLRANK: [u64; 2] = [0x00000000FF000000, 0x000000FF00000000];

// A file and ~(H file)
pub const FILE: u64 = 0x0101_0101_0101_0101;
pub const NOTH: u64 = !(FILE << 7);

// rook attacks on rank
pub const WEST: [u64; 64] = init!(let mut idx = 0, idx, 0, ((1 << idx) - 1) & (0xFF << (idx & 56)));
pub const EAST: [u64; 64] = init!(let mut idx = 0, idx, 0, (1 << idx) ^ WEST[idx] ^ (0xFF << (idx & 56)));

// pawn attacks
pub const PATT: [[u64; 64]; 2] = [
    init!(let mut idx = 0, idx, 0, (((1 << idx) & !FILE) << 7) | (((1 << idx) & NOTH) << 9)),
    init!(let mut idx = 0, idx, 0, (((1 << idx) & !FILE) >> 9) | (((1 << idx) & NOTH) >> 7)),
];

// knight attacks
pub const NATT: [u64; 64] = init!(let mut idx = 0, idx, 0, {
    let n = 1 << idx;
    let h1 = ((n >> 1) & 0x7f7f7f7f7f7f7f7f) | ((n << 1) & 0xfefefefefefefefe);
    let h2 = ((n >> 2) & 0x3f3f3f3f3f3f3f3f) | ((n << 2) & 0xfcfcfcfcfcfcfcfc);
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
});

// king attacks
pub const KATT: [u64; 64] = init!(let mut idx = 0, idx, 0, {
    let mut k = 1 << idx;
    k |= (k << 8) | (k >> 8);
    k |= ((k & !FILE) >> 1) | ((k & NOTH) << 1);
    k ^ (1 << idx)
});

// hyperbola quintessence rook and bishop attacks
#[derive(Clone, Copy)]
pub struct Mask {
    pub bit: u64,
    pub diag: u64,
    pub anti: u64,
    pub file: u64,
}

// diagonals
pub const DIAGS: [u64; 15] = [
    0x0100000000000000, 0x0201000000000000, 0x0402010000000000, 0x0804020100000000, 0x1008040201000000,
    0x2010080402010000, 0x4020100804020100, 0x8040201008040201, 0x0080402010080402, 0x0000804020100804,
    0x0000008040201008, 0x0000000080402010, 0x0000000000804020, 0x0000000000008040, 0x0000000000000080,
];

pub const MASKS: [Mask; 64] = init!(let mut idx = 0, idx, Mask { bit: 0, diag: 0, anti: 0, file: 0 }, {
    let bit = 1 << idx;
    Mask { bit, diag: bit ^ DIAGS[(7 + (idx & 7) - (idx >> 3))], anti: bit ^ DIAGS[((idx & 7) + (idx >> 3))].swap_bytes(), file: bit ^ FILE << (idx & 7) }
});
