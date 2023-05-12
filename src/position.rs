use super::consts::*;

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub bb: [u64; 8],
    pub c: bool,
    pub enp: u8,
    pub cr: u8,
}

#[derive(Copy, Clone, Default)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flag: u8,
    pub mpc: u8,
}

#[inline(always)]
pub fn batt(idx: usize, occ: u64) -> u64 {
    // hyperbola quintessence
    // this gets automatically vectorised when targeting avx or better
    // m.file = m.bit.swap_bytes() here, would be a spare field otherwise

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
pub fn ratt(idx: usize, occ: u64) -> u64 {
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

impl Position {
    #[inline(always)]
    pub fn toggle(&mut self, c: usize, pc: usize, bit: u64) {
        self.bb[pc] ^= bit;
        self.bb[c] ^= bit;
    }

    #[inline(always)]
    pub fn is_sq_att(&self, sq: usize, side: usize, occ: u64) -> bool {
        ( (NATT[sq] & self.bb[N])
        | (KATT[sq] & self.bb[K])
        | (PATT[side][sq] & self.bb[P])
        | (ratt(sq, occ) & (self.bb[R] | self.bb[Q]))
        | (batt(sq, occ) & (self.bb[B] | self.bb[Q]))
        ) & self.bb[side ^ 1] > 0
    }

    #[inline(always)]
    pub fn get_pc(&self, bit: u64) -> usize {
        usize::from((self.bb[N] | self.bb[R]) & bit > 0)
        | (2 * usize::from((self.bb[N] | self.bb[P] | self.bb[Q]) & bit > 0))
        | (4 * usize::from((self.bb[B] | self.bb[R] | self.bb[Q]) & bit > 0))
    }

    pub fn do_move(&mut self, m: Move) -> bool {
        // extracting move info
        let f = 1 << m.from;
        let t = 1 << m.to;
        let cpc = if m.flag & CAP == 0 { E } else { self.get_pc(t) };
        let side = usize::from(self.c);

        // updating state
        self.c = !self.c;
        self.enp = 0;
        self.cr &= CR[usize::from(m.to)] & CR[usize::from(m.from)];

        // move piece
        self.toggle(side, usize::from(m.mpc), f | t);

        // captures
        if cpc != E { self.toggle(side ^ 1, cpc, t) }

        // more complex moves
        match m.flag {
            DBL => self.enp = if side == WH { m.to - 8 } else { m.to + 8 },
            KS | QS => self.toggle(side, R, CM[usize::from(m.flag == KS)][side]),
            ENP => self.toggle(side ^ 1, P, 1 << (m.to.wrapping_add([8u8.wrapping_neg(), 8u8][side]))),
            NPR.. => {
                self.bb[P] ^= t;
                self.bb[usize::from((m.flag & 3) + 3)] ^= t;
            }
            _ => {}
        }

        // is move legal?
        let king_sq = (self.bb[K] & self.bb[side]).trailing_zeros();
        self.is_sq_att(king_sq as usize, side, self.bb[0] | self.bb[1])
    }
}
