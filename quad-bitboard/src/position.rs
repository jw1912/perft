use super::*;

#[derive(Copy, Clone)]
pub struct Pos {
    pub qbb: Qbb,
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
    // disclaimer: in BMASKS, m.file = m.bit.swap_bytes(), as the file isn't needed
    // hyperbola quintessence diagonal attacks
    let m: Mask = BMASKS[idx];
    let mut f: u64 = occ & m.right;
    let mut r: u64 = f.swap_bytes();
    f -= m.bit;
    r -= m.file;
    f ^= r.swap_bytes();
    f &= m.right;
    // hyperbola quintessence antidiagonal attacks
    let mut f2: u64 = occ & m.left;
    r = f2.swap_bytes();
    f2 -= m.bit;
    r -= m.file;
    f2 ^= r.swap_bytes();
    f2 &= m.left;

    f | f2
}

#[inline(always)]
pub fn ratt(idx: usize, occ: u64) -> u64 {
    // hyperbola quintessence file attacks
    let m: Mask = RMASKS[idx];
    let mut f: u64 = occ & m.file;
    let mut r: u64 = f.swap_bytes();
    f -= m.bit;
    r -= m.bit.swap_bytes();
    f ^= r.swap_bytes();
    f &= m.file;
    // subtracting a rook from a blocking piece eastward attacks
    let mut e: u64 = m.right & occ;
    r = e & e.wrapping_neg();
    e = (r ^ (r - m.bit)) & m.right;
    // classical westward attacks
    let w: u64 = m.left ^ WEST[(((m.left & occ)| 1).leading_zeros() ^ 63) as usize];

    f | e | w
}

impl Pos {
    pub fn do_move(&mut self, m: Move) -> bool {
        // extracting move info
        let f: Qbb = Qbb::splat(u64::from(m.from));
        let t: Qbb = Qbb::splat(u64::from(m.to));
        let mpc: Qbb = (self.qbb >> f) & qbb!(1, 1, 1, 1);
        let side: usize = usize::from(self.c);

        // updating board
        self.c = !self.c;
        self.enp = 0;
        self.cr &= CR[usize::from(m.to)] & CR[usize::from(m.from)];
        self.qbb &= !((qbb!(1, 1, 1, 1) << t) ^ (qbb!(1, 1, 1, 1) << f));
        self.qbb ^= mpc << t;
        match m.flag {
            QUIET => {},
            DBL => self.enp = if side == WH {m.to - 8} else {m.to + 8},
            KS => self.qbb ^= CKM[side],
            QS => self.qbb ^= CQM[side],
            ENP => self.qbb ^= qbb!(u64::from(self.c), 0, 0, 1) << Qbb::splat(u64::from(if self.c {m.to - 8} else {m.to + 8})),
            PROMO.. => self.qbb ^= PROMOS[usize::from(m.flag & 3)] << t,
        }

        // is move legal?
        let p: ExtPos = ExtPos::new(self, side);
        is_sq_att((p.k & p.f).trailing_zeros() as usize, side, &p)
    }
}
