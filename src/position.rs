use super::*;

macro_rules! msb {($x:expr, $t:ty) => {63 ^ $x.leading_zeros() as $t}}
macro_rules! from {($m:expr) => {(($m >> 6) & 63) as usize}}
macro_rules! to {($m:expr) => {($m & 63) as usize}}
macro_rules! bit {($x:expr) => {1 << $x}}

#[inline(always)]
pub fn batt(idx: usize, occ: u64) -> u64 {
    let m: Mask = MASKS[idx];
    let mut f: u64 = occ & m.diag;
    let mut r: u64 = f.swap_bytes();
    f -= m.bitmask;
    r -= m.bitmask.swap_bytes();
    f ^= r.swap_bytes();
    f &= m.diag;
    let mut f2: u64 = occ & m.antidiag;
    r = f2.swap_bytes();
    f2 -= m.bitmask;
    r -= m.bitmask.swap_bytes();
    f2 ^= r.swap_bytes();
    f2 &= m.antidiag;
    f | f2
}

#[inline(always)]
pub fn ratt(idx: usize, occ: u64) -> u64 {
    let m: Mask = MASKS[idx];
    let mut f: u64 = occ & m.file;
    let mut r: u64 = f.swap_bytes();
    f -= m.bitmask;
    r -= m.bitmask.swap_bytes();
    f ^= r.swap_bytes();
    f &= m.file;
    let mut e: u64 = EA[idx];
    let mut sq: usize = lsb!((e & occ) | MSB, usize);
    e ^= EA[sq];
    let mut w: u64 = WE[idx];
    sq = msb!((w & occ)| LSB, usize);
    w ^= WE[sq];
    f | e | w
}

impl Pos {
    #[inline(always)]
    pub fn toggle(&mut self, side: usize, pc: usize, bit: u64) {
        self.pc[pc] ^= bit;
        self.s[side] ^= bit;
    }

    #[inline(always)]
    pub fn is_sq_att(&self, idx: usize, side: usize, occ: u64) -> bool {
        let other: usize = side ^ 1;
        let s: u64 = self.s[other];
        let opp_queen: u64 = self.pc[Q] & s;
        (NATT[idx] & self.pc[N] & s > 0)
        || (KATT[idx] & self.pc[K] & s > 0)
        || (PATT[side][idx] & self.pc[P] & s > 0)
        || (ratt(idx, occ) & (self.pc[R] & s | opp_queen) > 0)
        || (batt(idx, occ) & (self.pc[B] & s | opp_queen) > 0)
    }

    #[inline(always)]
    pub fn get_pc(&self, bit: u64) -> usize {
        (self.pc[N] & bit > 0) as usize
        + B * (self.pc[B] & bit > 0) as usize
        + R * (self.pc[R] & bit > 0) as usize
        + Q * (self.pc[Q] & bit > 0) as usize
        + K * (self.pc[K] & bit > 0) as usize
        + E * (!(self.s[0] | self.s[1]) & bit > 0) as usize
    }

    pub fn do_move(&mut self, m: u16) -> bool {
        let (from, to): (usize, usize) = (from!(m), to!(m));
        let (f, t): (u64, u64) = (bit!(from), bit!(to));
        let (mpc, cpc): (usize, usize) = (self.get_pc(f), self.get_pc(t));
        let flag: u16 = m & 0xF000;
        let opp: usize = self.c ^ 1;

        self.toggle(self.c, mpc, f | t);
        self.state.enp = 0;
        if cpc != E { self.toggle(opp, cpc, t); }
        if cpc == R { self.state.cr &= CR[to]; }
        match mpc {
            P => {
                if flag == ENP {
                    let p: u64 = if opp == WH {t << 8} else {t >> 8};
                    self.toggle(opp, P, p);
                } else if flag == DBL {
                    self.state.enp = if self.c == WH {to - 8} else {to + 8} as u16;
                } else if flag >= PROMO {
                    let ppc: u16 = ((flag >> 12) & 3) + 1;
                    self.pc[mpc] ^= t;
                    self.pc[ppc as usize] ^= t;
                }
            }
            K => {
                self.state.cr &= CR[from];
                if flag == KS || flag == QS {
                    let c: u64 = CASTLE_MOVES[self.c][(flag == KS) as usize];
                    self.toggle(self.c, R, c);
                }
            }
            R => self.state.cr &= CR[from],
            _ => {}
        }
        self.state.hfm = (mpc > P && flag != CAP) as u8 * (self.state.hfm + 1);
        self.c ^= 1;

        let king_idx: usize = lsb!(self.pc[K] & self.s[opp ^ 1], usize);
        self.is_sq_att(king_idx, opp ^ 1, self.s[0] | self.s[1])
    }
}
