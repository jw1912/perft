use super::init;

pub struct Attacks;
impl Attacks {
    pub const PAWN: [[u64; 64]; 2] = [
        init! {idx, (((1 << idx) & !File::A) << 7) | (((1 << idx) & !File::H) << 9)},
        init! {idx, (((1 << idx) & !File::A) >> 9) | (((1 << idx) & !File::H) >> 7)},
    ];

    pub const KNIGHT: [u64; 64] = init! {idx, {
        let n = 1 << idx;
        let h1 = ((n >> 1) & 0x7f7f7f7f7f7f7f7f) | ((n << 1) & 0xfefefefefefefefe);
        let h2 = ((n >> 2) & 0x3f3f3f3f3f3f3f3f) | ((n << 2) & 0xfcfcfcfcfcfcfcfc);
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
    // m.file = m.bit.swap_bytes() here, would be a spare field otherwise
    #[inline(always)]
    pub fn bishop(idx: usize, occ: u64) -> u64 {
        // diagonal
        let m = BMASKS[idx];
        let mut f = occ & m.right;
        let mut r = f.swap_bytes();
        f = f.wrapping_sub(m.bit);
        r = r.wrapping_sub(m.file);
        f ^= r.swap_bytes();
        f &= m.right;

        // antidiagonal
        let mut f2 = occ & m.left;
        r = f2.swap_bytes();
        f2 = f2.wrapping_sub(m.bit);
        r = r.wrapping_sub(m.file);
        f2 ^= r.swap_bytes();
        f2 &= m.left;

        f | f2
    }

    #[inline(always)]
    pub fn rook(idx: usize, occ: u64) -> u64 {
        // hyperbola quintessence file attacks
        let m = RMASKS[idx];
        let mut f = occ & m.file;
        let mut r = f.swap_bytes();
        f = f.wrapping_sub(m.bit);
        r = r.wrapping_sub(m.bit.swap_bytes());
        f ^= r.swap_bytes();
        f &= m.file;

        // shift-lookup
        let file = idx & 7;
        let shift = idx - file;
        r = RANKS[file][((occ >> (shift + 1)) & 0x3F) as usize] << shift;

        f | r
    }

    #[inline(always)]
    pub fn queen(idx: usize, occ: u64) -> u64 {
        Self::bishop(idx, occ) | Self::rook(idx, occ)
    }
}

// A file and ~(H file)
struct File;
impl File {
    const A: u64 = 0x0101010101010101;
    const H: u64 = Self::A << 7;
}

const EAST: [u64; 64] = init! {idx, (1 << idx) ^ WEST[idx] ^ (0xFF << (idx & 56))};
const WEST: [u64; 64] = init! {idx, ((1 << idx) - 1) & (0xFF << (idx & 56))};
const DIAGS: [u64; 15] = [
    0x0100000000000000,
    0x0201000000000000,
    0x0402010000000000,
    0x0804020100000000,
    0x1008040201000000,
    0x2010080402010000,
    0x4020100804020100,
    0x8040201008040201,
    0x0080402010080402,
    0x0000804020100804,
    0x0000008040201008,
    0x0000000080402010,
    0x0000000000804020,
    0x0000000000008040,
    0x0000000000000080,
];

// masks for hyperbola quintessence bishop attacks
const BMASKS: [Mask; 64] = init! {idx,
    let bit = 1 << idx;
    Mask {
        bit,
        right: bit ^ DIAGS[7 + (idx & 7) - (idx >> 3)],
        left: bit ^ DIAGS[(idx & 7) + (idx >> 3)].swap_bytes(),
        file: bit.swap_bytes()
    }
};

// masks for hyperbola quintessence rook file attacks
const RMASKS: [Mask; 64] = init! {idx,
    Mask {
        bit: 1 << idx,
        right: EAST[idx],
        left: WEST[idx],
        file: (1 << idx) ^ File::A << (idx & 7)
    }
};

#[derive(Clone, Copy)]
struct Mask {
    bit: u64,
    right: u64,
    left: u64,
    file: u64,
}

// rank lookup for rook attacks
const RANKS: [[u64; 64]; 8] = {
    let mut ret = [[0; 64]; 8];
    let mut file = 0;
    while file < 8 {
        ret[file] = init! {idx, {
            let occ = (idx << 1) as u64;
            let east_idx = ((EAST[file] & occ) | (1 << 63)).trailing_zeros();
            let west_idx = ((WEST[file] & occ) | 1).leading_zeros() ^ 63;
            let east = EAST[file] ^ EAST[east_idx as usize];
            let west = WEST[file] ^ WEST[west_idx as usize];
            east | west
        }};
        file += 1;
    }
    ret
};