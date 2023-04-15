use super::Qbb;

/// Macro for calculating tables (until const fn pointers are stable).
macro_rules! init {
    ($idx:ident, $init:expr, $($rest:tt)+) => {{
        let mut res = [$init; 64];
        let mut $idx = 0;
        while $idx < 64 {
            res[$idx] = {$($rest)+};
            $idx += 1;
        }
        res
    }};
}

// macro creates a set of constants similar to in a C enum, but with a strict type and starts at a given value
macro_rules! c_enum {
    ($type:ty, $val:expr, $name:ident) => {pub const $name: $type = $val;};
    ($type:ty, $val:expr, $name:ident, $($b:tt),*) => {pub const $name: $type = $val; c_enum!($type, $val + 1, $($b),*);}
}

#[macro_export]
macro_rules! qbb {($v1:expr, $v2:expr, $v3:expr, $v4:expr) => {Qbb::from_array([$v1, $v2, $v3, $v4])}}

// pieces, sides and moveflags
c_enum!(usize, 0, WH, BL);
c_enum!(u8, 0, _E, P, N, B, R, Q, K);
c_enum!(u8, 0, QUIET, DBL, KS, QS, ENP, PROMO, BPROMO, RPROMO, QPROMO);

// castling
pub const WQS: u8 = 0b1000;
pub const WKS: u8 = 0b0100;
pub const BQS: u8 = 0b0010;
pub const BKS: u8 = 0b0001;
pub const CKM: [Qbb; 2] = [qbb!(0, 160, 0, 0), qbb!(0xA000000000000000, 0xA000000000000000, 0, 0)];
pub const CQM: [Qbb; 2] = [qbb!(0, 9, 0, 0), qbb!(0x0900000000000000, 0x0900000000000000, 0, 0)];
pub const B1C1D1: u64 = 0x000000000000000E;
pub const   F1G1: u64 = 0x0000000000000060;
pub const B8C8D8: u64 = 0x0E00000000000000;
pub const   F8G8: u64 = 0x6000000000000000;
pub const CS: [u8; 2] = [WKS | WQS, BKS | BQS];
pub const CR: [u8; 64] = init!(idx, 0, match idx {0 => 7, 4 => 3, 7 => 11, 56 => 13, 60 => 12, 63 => 14, _ => 15});

// for promotions / double pushes
pub const PENRANK: [u64; 2] = [0x00FF000000000000, 0x000000000000FF00];
pub const DBLRANK: [u64; 2] = [0x00000000FF000000, 0x000000FF00000000];
pub const PROMOS: [Qbb; 4] = [qbb!(0, 1, 0, 1), qbb!(0, 0, 1, 0), qbb!(0, 0, 1, 1), qbb!(0, 1, 0, 0)];

// A file and ~(H file)
pub const FILE: u64 = 0x0101010101010101;
pub const NOTH: u64 = !(FILE << 7);

// rook attacks on rank
pub const WEST: [u64; 64] = init!(idx, 0, ((1 << idx) - 1) & (0xFF << (idx & 56)));

// pawn attacks
pub const PATT: [[u64; 64]; 2] = [
    init!(idx, 0, (((1 << idx) & !FILE) << 7) | (((1 << idx) & NOTH) << 9)),
    init!(idx, 0, (((1 << idx) & !FILE) >> 9) | (((1 << idx) & NOTH) >> 7)),
];

// knight attacks
pub const NATT: [u64; 64] = init!(idx, 0, {
    let n = 1 << idx;
    let h1 = ((n >> 1) & 0x7f7f7f7f7f7f7f7f) | ((n << 1) & 0xfefefefefefefefe);
    let h2 = ((n >> 2) & 0x3f3f3f3f3f3f3f3f) | ((n << 2) & 0xfcfcfcfcfcfcfcfc);
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
});

// king attacks
pub const KATT: [u64; 64] = init!(idx, 0, {
    let mut k = 1 << idx;
    k |= (k << 8) | (k >> 8);
    k |= ((k & !FILE) >> 1) | ((k & NOTH) << 1);
    k ^ (1 << idx)
});

// diagonals
pub const DIAGS: [u64; 15] = [
    0x0100000000000000, 0x0201000000000000, 0x0402010000000000, 0x0804020100000000, 0x1008040201000000,
    0x2010080402010000, 0x4020100804020100, 0x8040201008040201, 0x0080402010080402, 0x0000804020100804,
    0x0000008040201008, 0x0000000080402010, 0x0000000000804020, 0x0000000000008040, 0x0000000000000080,
];

// masks for hyperbola quintessence rook and bishop attacks
pub const BMASKS: [Mask; 64] = init!(idx, Mask { bit: 0, right: 0, left: 0, file: 0 },
    let bit = 1 << idx;
    Mask { bit, right: bit ^ DIAGS[7 + (idx & 7) - (idx >> 3)], left: bit ^ DIAGS[(idx & 7) + (idx >> 3)].swap_bytes(), file: bit.swap_bytes() }
);

pub const RMASKS: [Mask; 64] = init!(idx, Mask { bit: 0, right: 0, left: 0, file: 0 },
    let bit = 1 << idx;
    let left = (bit - 1) & (0xFF << (idx & 56));
    Mask { bit, right: bit ^ left ^ (0xFF << (idx & 56)), left, file: bit ^ FILE << (idx & 7) }
);

#[derive(Clone, Copy)]
pub struct Mask {
    pub bit: u64,
    pub right: u64,
    pub left: u64,
    pub file: u64,
}
