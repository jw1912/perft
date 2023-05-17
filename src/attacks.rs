// Macro for calculating tables (until const fn pointers are stable).
#[macro_export]
macro_rules! init {
    ($idx:ident, $($rest:tt)+) => {{
        let mut $idx = 0;
        let mut res = [{$($rest)+}; 64];
        while $idx < 64 {
            res[$idx] = {$($rest)+};
            $idx += 1;
        }
        res
    }};
}

pub struct Attacks;
impl Attacks {
    pub const PAWN: [[u64; 64]; 2] = [
        init! {idx, (((1 << idx) & !File::A) << 7) | (((1 << idx) & !File::H) << 9)},
        init! {idx, (((1 << idx) & !File::A) >> 9) | (((1 << idx) & !File::H) >> 7)},
    ];

    pub const KNIGHT: [u64; 64] = init! {idx, {
        let n = 1 << idx;
        let h1 = ((n >> 1) & 0x7f7f_7f7f_7f7f_7f7f) | ((n << 1) & 0xfefe_fefe_fefe_fefe);
        let h2 = ((n >> 2) & 0x3f3f_3f3f_3f3f_3f3f) | ((n << 2) & 0xfcfc_fcfc_fcfc_fcfc);
        (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
    }};

    pub const KING: [u64; 64] = init! {idx, {
        let mut k = 1 << idx;
        k |= (k << 8) | (k >> 8);
        k |= ((k & !File::A) >> 1) | ((k & !File::H) << 1);
        k ^ (1 << idx)
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

        let flip = ((occ >> file) & File::A).wrapping_mul(LEADING_DIAG);
        let file_idx = (flip >> 57) & 0x3F;
        let files = Lookup::FILE[rank][file_idx as usize] >> (7 - file);

        let rank_shift = sq - file;
        let rank_idx = (occ >> (rank_shift + 1)) & 0x3F;
        let ranks = Lookup::RANK[file][rank_idx as usize] << rank_shift;

        ranks | files
    }

    #[inline]
    pub fn queen(idx: usize, occ: u64) -> u64 {
        Self::bishop(idx, occ) | Self::rook(idx, occ)
    }
}

// A file and ~(H file)
struct File;
impl File {
    const A: u64 = 0x0101_0101_0101_0101;
    const H: u64 = Self::A << 7;
}

const EAST: [u64; 64] = init! {idx, (1 << idx) ^ WEST[idx] ^ (0xFF << (idx & 56))};
const WEST: [u64; 64] = init! {idx, ((1 << idx) - 1) & (0xFF << (idx & 56))};
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
const LEADING_DIAG: u64 = DIAGS[7];

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
    const BISHOP: [Mask; 64] = init! {idx,
        let bit = 1 << idx;
        Mask {
            bit,
            diag: bit ^ DIAGS[7 + (idx & 7) - (idx >> 3)],
            anti: bit ^ DIAGS[(idx & 7) + (idx >> 3)].swap_bytes(),
            swap: bit.swap_bytes()
        }
    };

    const RANK: [[u64; 64]; 8] = {
        let mut ret = [[0; 64]; 8];
        let mut file = 0;
        while file < 8 {
            ret[file] = init! {idx, {
                let occ = (idx << 1) as u64;
                // classical attacks for the rank
                let east = ((EAST[file] & occ) | (1 << 63)).trailing_zeros() as usize;
                let west = ((WEST[file] & occ) | 1).leading_zeros() as usize ^ 63;
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
            ret[rank] = init! {idx, {
                let ranks = Self::RANK[7 - rank][idx];
                ranks.wrapping_mul(LEADING_DIAG) & File::H
            }};
            rank += 1;
        }
        ret
    };
}
