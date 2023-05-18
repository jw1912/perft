// Macro for calculating tables (until const fn pointers are stable).
#[macro_export]
macro_rules! init {
    ($sq:ident, $($rest:tt)+) => {{
        let mut $sq = 0;
        let mut res = [{$($rest)+}; 64];
        while $sq < 64 {
            res[$sq] = {$($rest)+};
            $sq += 1;
        }
        res
    }};
}

pub struct Attacks;
impl Attacks {
    pub const PAWN: [[u64; 64]; 2] = [
        init! {sq, (((1 << sq) & !File::A) << 7) | (((1 << sq) & !File::H) << 9)},
        init! {sq, (((1 << sq) & !File::A) >> 9) | (((1 << sq) & !File::H) >> 7)},
    ];

    pub const KNIGHT: [u64; 64] = init! {sq, {
        let n = 1 << sq;
        let h1 = ((n >> 1) & 0x7f7f_7f7f_7f7f_7f7f) | ((n << 1) & 0xfefe_fefe_fefe_fefe);
        let h2 = ((n >> 2) & 0x3f3f_3f3f_3f3f_3f3f) | ((n << 2) & 0xfcfc_fcfc_fcfc_fcfc);
        (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
    }};

    pub const KING: [u64; 64] = init! {sq, {
        let mut k = 1 << sq;
        k |= (k << 8) | (k >> 8);
        k |= ((k & !File::A) >> 1) | ((k & !File::H) << 1);
        k ^ (1 << sq)
    }};

    // hyperbola quintessence
    // this gets automatically vectorised when targeting avx or better
    #[inline]
    pub fn bishop(sq: usize, occ: u64) -> u64 {
        let mask = Lookup::BISHOP[sq];

        let mut diag = occ & mask.diag;
        let mut rev1 = diag.swap_bytes();
        diag  = diag.wrapping_sub(mask.bit);
        rev1  = rev1.wrapping_sub(mask.swap);
        diag ^= rev1.swap_bytes();
        diag &= mask.diag;

        let mut anti = occ & mask.anti;
        let mut rev2 = anti.swap_bytes();
        anti  = anti.wrapping_sub(mask.bit);
        rev2  = rev2.wrapping_sub(mask.swap);
        anti ^= rev2.swap_bytes();
        anti &= mask.anti;

        diag | anti
    }

    // shifted lookup
    // files and ranks are mapped to 1st rank and looked up by occupancy
    #[inline]
    pub fn rook(sq: usize, occ: u64) -> u64 {
        let file = sq & 7;
        let rank = sq / 8;

        let flip = ((occ >> file) & File::A).wrapping_mul(DIAG);
        let file_sq = (flip >> 57) & 0x3F;
        let files = Lookup::FILE[rank][file_sq as usize] >> (7 - file);

        let rank_shift = sq - file;
        let rank_sq = (occ >> (rank_shift + 1)) & 0x3F;
        let ranks = Lookup::RANK[file][rank_sq as usize] << rank_shift;

        ranks | files
    }

    #[inline]
    pub fn queen(sq: usize, occ: u64) -> u64 {
        Self::bishop(sq, occ) | Self::rook(sq, occ)
    }
}

struct File;
impl File {
    const A: u64 = 0x0101_0101_0101_0101;
    const H: u64 = Self::A << 7;
}

const EAST: [u64; 64] = init! {sq, (0xFF << (sq & 56)) ^ (1 << sq) ^ WEST[sq]};
const WEST: [u64; 64] = init! {sq, (0xFF << (sq & 56)) & ((1 << sq) - 1)};
const DIAG: u64 = DIAGS[7];
const DIAGS: [u64; 15] = [
    0x0100_0000_0000_0000,
    0x0201_0000_0000_0000,
    0x0402_0100_0000_0000,
    0x0804_0201_0000_0000,
    0x1008_0402_0100_0000,
    0x2010_0804_0201_0000,
    0x4020_1008_0402_0100,
    0x8040_2010_0804_0201,
    0x0080_4020_1008_0402,
    0x0000_8040_2010_0804,
    0x0000_0080_4020_1008,
    0x0000_0000_8040_2010,
    0x0000_0000_0080_4020,
    0x0000_0000_0000_8040,
    0x0000_0000_0000_0080,
];

// masks for hyperbola quintessence bishop attacks
#[derive(Clone, Copy)]
struct Mask {
    bit: u64,
    diag: u64,
    anti: u64,
    swap: u64,
}

struct Lookup;
impl Lookup {
    const BISHOP: [Mask; 64] = init! {sq,
        let bit = 1 << sq;
        let file = sq & 7;
        let rank = sq / 8;
        Mask {
            bit,
            diag: bit ^ DIAGS[7 + file - rank],
            anti: bit ^ DIAGS[    file + rank].swap_bytes(),
            swap: bit.swap_bytes()
        }
    };

    const RANK: [[u64; 64]; 8] = {
        let mut ret = [[0; 64]; 8];
        let mut file = 0;
        while file < 8 {
            ret[file] = init! {occ, {
                let mask = (occ << 1) as u64;
                let east = ((EAST[file] & mask) | (1 << 63)).trailing_zeros() as usize;
                let west = ((WEST[file] & mask) | 1).leading_zeros() as usize ^ 63;
                EAST[file] ^ EAST[east] | WEST[file] ^ WEST[west]
            }};
            file += 1;
        }
        ret
    };

    const FILE: [[u64; 64]; 8] = {
        let mut ret = [[0; 64]; 8];
        let mut rank = 0;
        while rank < 8 {
            ret[rank] = init! {occ, {
                let ranks = Self::RANK[7 - rank][occ];
                ranks.wrapping_mul(DIAG) & File::H
            }};
            rank += 1;
        }
        ret
    };
}
