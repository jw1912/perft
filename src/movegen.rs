use super::{*, position::{ratt, batt}};

macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u8; $x &= $x - 1}}

#[inline(always)]
fn encode<const PC: usize, const FLAG: u8>(moves: &mut MoveList, mut attacks: u64, from: u8) {
    let mut aidx: u8;
    while attacks > 0 {
        pop_lsb!(aidx, attacks);
        moves.push(from, aidx, FLAG, PC as u8);
    }
}

impl Pos {
    #[inline(always)]
    pub fn gen(&self, moves: &mut MoveList) {
        let occ: u64 = self.s[0] | self.s[1];
        let friends: u64 = self.s[self.c];
        let opps: u64 = self.s[self.c ^ 1];
        let pawns: u64 = self.pc[P] & friends;
        if self.c == WH {pawn_pushes::<WH>(moves, occ, pawns)} else {pawn_pushes::<BL>(moves, occ, pawns)}
        pawn_captures(moves, pawns, self.s[self.c ^ 1], self.c);
        if self.state.enp > 0 {en_passants(moves, pawns, self.state.enp, self.c)}
        pc_moves::<N>(moves, occ, friends, opps, self.pc[N]);
        pc_moves::<B>(moves, occ, friends, opps, self.pc[B]);
        pc_moves::<R>(moves, occ, friends, opps, self.pc[R]);
        pc_moves::<Q>(moves, occ, friends, opps, self.pc[Q]);
        pc_moves::<K>(moves, occ, friends, opps, self.pc[K]);
        if self.state.cr & SIDES[self.c] > 0 && !self.is_sq_att(4 + 56 * (self.c == BL) as usize, self.c, occ) {self.castles(moves, occ)}
    }

    fn castles(&self, moves: &mut MoveList, occ: u64) {
        let r = self.state.cr;
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

fn pc_moves<const PC: usize>(moves: &mut MoveList, occ: u64, friends: u64, opps: u64, mut attackers: u64) {
    let mut from: u8;
    let mut idx: usize;
    let mut attacks: u64;
    attackers &= friends;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        idx = from as usize;
        attacks = match PC {
            N => NATT[idx],
            R => ratt(idx, occ),
            B => batt(idx, occ),
            Q => ratt(idx, occ) | batt(idx, occ),
            K => KATT[idx],
            _ => 0,
        };
        encode::<PC, CAP>(moves, attacks & opps, from);
        encode::<PC, QUIET>(moves, attacks & !occ, from);
    }
}

fn pawn_captures(moves: &mut MoveList, mut attackers: u64, opps: u64, c: usize) {
    let (mut from, mut cidx): (u8, u8);
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
            pop_lsb!(cidx, attacks);
            moves.push(from, cidx, QPROMO_CAP, P as u8);
            moves.push(from, cidx, PROMO_CAP , P as u8);
            moves.push(from, cidx, BPROMO_CAP, P as u8);
            moves.push(from, cidx, RPROMO_CAP, P as u8);
        }
    }
}

fn en_passants(moves: &mut MoveList, pawns: u64, sq: u8, c: usize) {
    let mut attackers: u64 = PATT[c ^ 1][sq as usize] & pawns;
    let mut cidx: u8;
    while attackers > 0 {
        pop_lsb!(cidx, attackers);
        moves.push(cidx, sq, ENP, P as u8);
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

fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occupied: u64, pawns: u64) {
    let empty: u64 = !occupied;
    let mut pushable_pawns: u64 = shift(empty, SIDE) & pawns;
    let mut dbl_pushable_pawns: u64 = shift(shift(empty & DBLRANK[SIDE], SIDE) & empty, SIDE) & pawns;
    let mut promotable_pawns: u64 = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    let mut idx: u8;
    while pushable_pawns > 0 {
        pop_lsb!(idx, pushable_pawns);
        moves.push(idx, idx_shift::<8>(idx, SIDE), QUIET, P as u8);
    }
    while promotable_pawns > 0 {
        pop_lsb!(idx, promotable_pawns);
        let to: u8 = idx_shift::<8>(idx, SIDE);
        moves.push(idx, to, QPROMO, P as u8);
        moves.push(idx, to, PROMO , P as u8);
        moves.push(idx, to, BPROMO, P as u8);
        moves.push(idx, to, RPROMO, P as u8);
    }
    while dbl_pushable_pawns > 0 {
        pop_lsb!(idx, dbl_pushable_pawns);
        moves.push(idx, idx_shift::<16>(idx, SIDE), DBL, P as u8);
    }
}
