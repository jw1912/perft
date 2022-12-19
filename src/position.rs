use super::*;

#[derive(Copy, Clone)]
pub struct Pos {
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
    let m: Mask = MASKS[idx];
    let mut f: u64 = occ & m.diag;
    let mut r: u64 = f.swap_bytes();
    f -= m.bit;
    r -= m.bit.swap_bytes();
    f ^= r.swap_bytes();
    f &= m.diag;
    let mut f2: u64 = occ & m.anti;
    r = f2.swap_bytes();
    f2 -= m.bit;
    r -= m.bit.swap_bytes();
    f2 ^= r.swap_bytes();
    f2 &= m.anti;
    f | f2
}

#[inline(always)]
pub fn ratt(idx: usize, occ: u64) -> u64 {
    let m: Mask = MASKS[idx];
    let mut f: u64 = occ & m.file;
    let mut r: u64 = f.swap_bytes();
    f -= m.bit;
    r -= m.bit.swap_bytes();
    f ^= r.swap_bytes();
    f &= m.file;
    let mut e: u64 = EAST[idx];
    let mut sq: usize = ((e & occ) | MSB).trailing_zeros() as usize;
    e ^= EAST[sq];
    let mut w: u64 = WEST[idx];
    sq = (((w & occ)| LSB).leading_zeros() ^ 63) as usize;
    w ^= WEST[sq];
    f | e | w
}

impl Pos {
    #[inline(always)]
    pub fn toggle(&mut self, c: usize, pc: usize, bit: u64) {
        self.bb[pc] ^= bit;
        self.bb[c] ^= bit;
    }

    #[inline(always)]
    pub fn is_sq_att(&self, idx: usize, side: usize, occ: u64) -> bool {
        let s: u64 = self.bb[side ^ 1];
        (NATT[idx] & self.bb[N] & s > 0)
        || (KATT[idx] & self.bb[K] & s > 0)
        || (PATT[side][idx] & self.bb[P] & s > 0)
        || (ratt(idx, occ) & ((self.bb[R] | self.bb[Q]) & s) > 0)
        || (batt(idx, occ) & ((self.bb[B] | self.bb[Q]) & s) > 0)
    }

    #[inline(always)]
    pub fn get_pc(&self, bit: u64) -> usize {
        ((self.bb[N] | self.bb[R] | self.bb[K]) & bit > 0) as usize
        | (2 * ((self.bb[P] | self.bb[N] | self.bb[Q] | self.bb[K]) & bit > 0) as usize)
        | (4 * ((self.bb[B] | self.bb[R] | self.bb[Q] | self.bb[K]) & bit > 0) as usize)
    }

    pub fn do_move(&mut self, m: Move) -> bool {
        // extracting move info
        let f: u64 = 1 << m.from;
        let t: u64 = 1 << m.to;
        let mpc: usize = m.mpc as usize;
        let cpc: usize = if m.flag & CAP == 0 || m.flag == ENP {E} else {self.get_pc(t)};

        // flipping side to move
        let side: usize = usize::from(self.c);
        self.c = !self.c;

        // en passant and castling rights
        self.enp = 0;
        if cpc == R { self.cr &= CR[m.to as usize] }
        if mpc == R || mpc == K { self.cr &= CR[m.from as usize] }

        // updating board
        self.toggle(side, mpc, f | t);
        if cpc != E { self.toggle(side ^ 1, cpc, t) }
        match m.flag {
            DBL => self.enp = if side == WH {m.to - 8} else {m.to + 8},
            KS => self.toggle(side, R, CKM[side]),
            QS => self.toggle(side, R, CQM[side]),
            ENP => self.toggle(side ^ 1, P, if side == BL {t << 8} else {t >> 8}),
            PROMO.. => {
                self.bb[mpc] ^= t;
                self.bb[((m.flag & 3) + 3) as usize] ^= t;
            }
            _ => {}
        }

        // is move legal?
        let king_idx: usize = (self.bb[K] & self.bb[side]).trailing_zeros() as usize;
        self.is_sq_att(king_idx, side, self.bb[0] | self.bb[1])
    }
}
