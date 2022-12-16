use super::{*, position::{ratt, batt}};

macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u8; $x &= $x - 1}}

pub struct MoveList {
    pub list: [Move; 252],
    pub len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {list: [Move::default(); 252], len: 0}
    }
}
impl MoveList {
    #[inline(always)]
    pub fn push(&mut self, from: u8, to: u8, flag: u8, mpc: u8) {
        self.list[self.len] = Move {from, to, flag, mpc};
        self.len += 1;
    }
}

#[inline(always)]
fn encode<const PC: usize, const FLAG: u8>(moves: &mut MoveList, mut attacks: u64, from: u8) {
    let mut to: u8;
    while attacks > 0 {
        pop_lsb!(to, attacks);
        moves.push(from, to, FLAG, PC as u8);
    }
}

impl Pos {
    pub fn gen<const QUIETS: bool>(&self, moves: &mut MoveList) {
        let occ: u64 = self.s[0] | self.s[1];
        let friends: u64 = self.s[self.c];
        let opps: u64 = self.s[self.c ^ 1];
        let pawns: u64 = self.pc[P] & friends;
        if QUIETS {
            if self.cr & SIDES[self.c] > 0 && !self.is_sq_att(4 + 56 * (self.c == BL) as usize, self.c, occ) {self.castles(moves, occ)}
            if self.c == WH {pawn_pushes::<WH>(moves, occ, pawns);} else {pawn_pushes::<BL>(moves, occ, pawns);}
        }
        pawn_captures(moves, pawns, opps, self.c);
        if self.enp > 0 {en_passants(moves, pawns, self.enp, self.c)}
        pc_moves::<N, QUIETS>(moves, occ, friends, opps, self.pc[N]);
        pc_moves::<B, QUIETS>(moves, occ, friends, opps, self.pc[B]);
        pc_moves::<R, QUIETS>(moves, occ, friends, opps, self.pc[R]);
        pc_moves::<Q, QUIETS>(moves, occ, friends, opps, self.pc[Q]);
        pc_moves::<K, QUIETS>(moves, occ, friends, opps, self.pc[K]);
    }

    #[inline(always)]
    fn castles(&self, moves: &mut MoveList, occ: u64) {
        let r = self.cr;
        if self.c == WH {
            if r & WQS > 0 && occ & B1C1D1 == 0 && !self.is_sq_att(3, WH, occ) {
                moves.push(4, 2, QS, K as u8);
            }
            if r & WKS > 0 && occ & F1G1 == 0 && !self.is_sq_att(5, WH, occ) {
                moves.push(4, 6, KS, K as u8);
            }
        } else {
            if r & BQS > 0 && occ & B8C8D8 == 0 && !self.is_sq_att(59, BL, occ) {
                moves.push(60, 58, QS, K as u8);
            }
            if r & BKS > 0 && occ & F8G8 == 0 && !self.is_sq_att(61, BL, occ) {
                moves.push(60, 62, KS, K as u8);
            }
        }
    }
}

#[inline(always)]
fn pc_moves<const PC: usize, const QUIETS: bool>(moves: &mut MoveList, occ: u64, friends: u64, opps: u64, mut attackers: u64) {
    let mut from: u8;
    let mut attacks: u64;
    attackers &= friends;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = match PC {
            N => NATT[from as usize],
            R => ratt(from as usize, occ),
            B => batt(from as usize, occ),
            Q => ratt(from as usize, occ) | batt(from as usize, occ),
            K => KATT[from as usize],
            _ => 0,
        };
        encode::<PC, CAP>(moves, attacks & opps, from);
        if QUIETS { encode::<PC, QUIET>(moves, attacks & !occ, from); }
    }
}

#[inline(always)]
fn pawn_captures(moves: &mut MoveList, mut attackers: u64, opps: u64, c: usize) {
    let (mut from, mut to): (u8, u8);
    let mut attacks: u64;
    let mut promo_attackers: u64 = attackers & PENRANK[c];
    attackers &= !PENRANK[c];
    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = PATT[c][from as usize] & opps;
        encode::<P, CAP>(moves, attacks, from);
    }
    while promo_attackers > 0 {
        pop_lsb!(from, promo_attackers);
        attacks = PATT[c][from as usize] & opps;
        while attacks > 0 {
            pop_lsb!(to, attacks);
            moves.push(from, to, QPROMO_CAP, P as u8);
            moves.push(from, to, PROMO_CAP , P as u8);
            moves.push(from, to, BPROMO_CAP, P as u8);
            moves.push(from, to, RPROMO_CAP, P as u8);
        }
    }
}

#[inline(always)]
fn en_passants(moves: &mut MoveList, pawns: u64, sq: u8, c: usize) {
    let mut attackers: u64 = PATT[c ^ 1][sq as usize] & pawns;
    let mut from: u8;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        moves.push(from, sq, ENP, P as u8);
    }
}

#[inline(always)]
fn shift(bb: u64, c: usize) -> u64 {
    if c == WH {bb >> 8} else {bb << 8}
}

#[inline(always)]
fn idx_shift<const AMOUNT: u8>(idx: u8, c: usize) -> u8 {
    if c == WH {idx + AMOUNT} else {idx - AMOUNT}
}

#[inline(always)]
fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occupied: u64, pawns: u64) {
    let empty: u64 = !occupied;
    let mut pushable_pawns: u64 = shift(empty, SIDE) & pawns;
    let mut dbl_pushable_pawns: u64 = shift(shift(empty & DBLRANK[SIDE], SIDE) & empty, SIDE) & pawns;
    let mut promotable_pawns: u64 = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    let mut from: u8;
    while pushable_pawns > 0 {
        pop_lsb!(from, pushable_pawns);
        moves.push(from, idx_shift::<8>(from, SIDE), QUIET, P as u8);
    }
    while promotable_pawns > 0 {
        pop_lsb!(from, promotable_pawns);
        let to: u8 = idx_shift::<8>(from, SIDE);
        moves.push(from, to, QPROMO, P as u8);
        moves.push(from, to, PROMO , P as u8);
        moves.push(from, to, BPROMO, P as u8);
        moves.push(from, to, RPROMO, P as u8);
    }
    while dbl_pushable_pawns > 0 {
        pop_lsb!(from, dbl_pushable_pawns);
        moves.push(from, idx_shift::<16>(from, SIDE), DBL, P as u8);
    }
}
