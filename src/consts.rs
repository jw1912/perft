// Macro for calculating tables (until const fn pointers are stable).
#[macro_export]
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

pub struct Side;
impl Side {
    pub const WHITE: usize = 0;
    pub const BLACK: usize = 1;
}

pub struct Piece;
impl Piece {
    pub const  EMPTY: usize = 0;
    pub const   PAWN: usize = 2;
    pub const KNIGHT: usize = 3;
    pub const BISHOP: usize = 4;
    pub const   ROOK: usize = 5;
    pub const  QUEEN: usize = 6;
    pub const   KING: usize = 7;
}

pub struct Flag;
impl Flag {
    pub const QUIET: u8 = 0;
    pub const DBL: u8 = 1;
    pub const  KS: u8 = 2;
    pub const  QS: u8 = 3;
    pub const CAP: u8 = 4;
    pub const ENP: u8 = 5;
    pub const NPR: u8 = 8;
    pub const BPR: u8 = 9;
    pub const RPR: u8 = 10;
    pub const QPR: u8 = 11;
    pub const NPC: u8 = 12;
    pub const BPC: u8 = 13;
    pub const RPC: u8 = 14;
    pub const QPC: u8 = 15;
}

// castle rights
pub struct Right;
impl Right {
    pub const WQS: u8 = 0b1000;
    pub const WKS: u8 = 0b0100;
    pub const BQS: u8 = 0b0010;
    pub const BKS: u8 = 0b0001;
    pub const SIDE: [u8; 2] = [Self::WKS | Self::WQS, Self::BKS | Self::BQS];
}

// path required to be clear for castling
pub struct Path;
impl Path {
    pub const BD1: u64 = 0x000000000000000E;
    pub const FG1: u64 = 0x0000000000000060;
    pub const BD8: u64 = 0x0E00000000000000;
    pub const FG8: u64 = 0x6000000000000000;
}

// for efficient move making
pub const CM: [[u64; 2]; 2] = [[9, 0x0900000000000000], [160, 0xA000000000000000]];
pub const CR: [u8; 64] =
    init! {idx, 0, match idx {0 => 7, 4 => 3, 7 => 11, 56 => 13, 60 => 12, 63 => 14, _ => 15}};

// for promotions / double pushes
pub const PENRANK: [u64; 2] = [0x00FF000000000000, 0x000000000000FF00];
pub const DBLRANK: [u64; 2] = [0x00000000FF000000, 0x000000FF00000000];
