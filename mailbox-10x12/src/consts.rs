/// Creates a set of constants similar to in a C enum, but with a strict type and starts at a given value.
macro_rules! c_enum {
    ($type:ty, $val:expr, $name:ident) => {pub const $name: $type = $val;};
    ($type:ty, $val:expr, $name:ident, $($b:tt),*) => {pub const $name: $type = $val; c_enum!($type, $val + 1, $($b),*);}
}

// Pieces, sides and moveflags.
c_enum!(usize, 0, WH, BL);
c_enum!(u8, 0, E, P, N, B, R, Q, K);
c_enum!(u16, 0, QUIET, DBL, KS, QS, ENP, PROMO, BPROMO, RPROMO, QPROMO);

// Conversion 8x8 array indices to 10x12 indices.
pub const MAILBOX_64: [u8; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28,
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98,
];

// Off board index.
pub const XX: u8 = 0xFF;

// Conversion 10x12 array indices to 8x8 indices.
pub const MAILBOX_120: [u8; 120] = [
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
    XX,  0,  1,  2,  3,  4,  5,  6,  7, XX,
    XX,  8,  9, 10, 11, 12, 13, 14, 15, XX,
    XX, 16, 17, 18, 19, 20, 21, 22, 23, XX,
    XX, 24, 25, 26, 27, 28, 29, 30, 31, XX,
    XX, 32, 33, 34, 35, 36, 37, 38, 39, XX,
    XX, 40, 41, 42, 43, 44, 45, 46, 47, XX,
    XX, 48, 49, 50, 51, 52, 53, 54, 55, XX,
    XX, 56, 57, 58, 59, 60, 61, 62, 63, XX,
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
];

// Castling.
pub const WQS: u8 = 0b1000;
pub const WKS: u8 = 0b0100;
pub const BQS: u8 = 0b0010;
pub const BKS: u8 = 0b0001;
pub const CS: [u8; 2] = [WKS | WQS, BKS | BQS];
pub const CQM: [(u16, u16); 2] = [(0, 3), (56, 59)];
pub const CKM: [(u16, u16); 2] = [(7, 5), (63, 61)];
pub const CR: [u8; 64] = [
     7, 15, 15, 15,  3, 15, 15, 11,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    13, 15, 15, 15, 12, 15, 15, 14,
];

pub const OFFSETS: [[i16; 8]; 7] = [
    [   0,   0,  0,  0, 0,  0,  0,  0 ],
	[   0,   0,  0,  0, 0,  0,  0,  0 ],
	[ -21, -19,-12, -8, 8, 12, 19, 21 ],
	[ -11,  -9,  9, 11, 0,  0,  0,  0 ],
	[ -10,  -1,  1, 10, 0,  0,  0,  0 ],
	[ -11, -10, -9, -1, 1,  9, 10, 11 ],
	[ -11, -10, -9, -1, 1,  9, 10, 11 ],
];

pub const NON_SLIDER: [bool; 7] = [true, true, true, false, false, false, true];

pub const DOUBLE_RANKS: [u8; 2] = [1, 6];
pub const PROMO_RANKS: [u8; 2] = [6, 1];

pub const PAWN_CAPS: [[i16; 2]; 2] = [[9, 11], [-11, -9]];
pub const PUSH: [i16; 2] = [10, -10];