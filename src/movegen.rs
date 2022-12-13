use super::{*, position::{ratt, batt}};

macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u16; $x &= $x - 1}}
macro_rules! push_move {($l:expr, $m:expr) => {$l.list[$l.len] = $m; $l.len += 1;}}

#[inline(always)]
fn encode(moves: &mut MoveList, mut attacks: u64, from: u16) {
    let f: u16 = from << 6;
    let mut aidx: u16;
    while attacks > 0 {
        pop_lsb!(aidx, attacks);
        push_move!(moves, f | aidx);
    }
}

impl Pos {
    pub fn gen(&self, moves: &mut MoveList) {
        let occ: u64 = self.s[0] | self.s[1];
        let friends: u64 = self.s[self.c];
        let pawns: u64 = self.pc[P] & friends;
        if self.c == WH {pawn_pushes::<WH>(moves, occ, pawns)} else {pawn_pushes::<BL>(moves, occ, pawns)}
        if self.state.cr & SIDES[self.c] > 0 && !self.is_sq_att(4 + 56 * (self.c == BL) as usize, self.c, occ) {self.castles(moves, occ)}
        pawn_captures(moves, pawns, self.s[self.c ^ 1], self.c);
        if self.state.enp > 0 {en_passants(moves, pawns, self.state.enp, self.c)}
        pc_moves::<N>(moves, occ, friends, self.pc[N]);
        pc_moves::<B>(moves, occ, friends, self.pc[B]);
        pc_moves::<R>(moves, occ, friends, self.pc[R]);
        pc_moves::<Q>(moves, occ, friends, self.pc[Q]);
        pc_moves::<K>(moves, occ, friends, self.pc[K]);
    }

    fn castles(&self, moves: &mut MoveList, occ: u64) {
        let r = self.state.cr;
        if self.c == WH {
            if r & WQS > 0 && occ & B1C1D1 == 0 && !self.is_sq_att(3, WH, occ) {
                push_move!(moves, QS | 2 | 4 << 6);
            }
            if r & WKS > 0 && occ & F1G1 == 0 && !self.is_sq_att(5, WH, occ) {
                push_move!(moves, KS | 6 | 4 << 6);
            }
        } else {
            if r & BQS > 0 && occ & B8C8D8 == 0 && !self.is_sq_att(59, BL, occ) {
                push_move!(moves, QS | 58 | 60 << 6);
            }
            if r & BKS > 0 && occ & F8G8 == 0 && !self.is_sq_att(61, BL, occ) {
                push_move!(moves, KS | 62 | 60 << 6);
            }
        }
    }
}

fn pc_moves<const PC: usize>(moves: &mut MoveList, occ: u64, friends: u64, mut attackers: u64) {
    let mut from: u16;
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
        encode(moves, attacks & !friends, from);
    }
}

fn pawn_captures(moves: &mut MoveList, mut attackers: u64, opps: u64, c: usize) {
    let (mut from, mut cidx, mut f): (u16, u16, u16);
    let mut attacks: u64;
    let mut promo_attackers: u64 = attackers & PENRANK[c];
    attackers &= !PENRANK[c];
    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = PATT[c][from as usize] & opps;
        encode(moves, attacks, from);
    }
    while promo_attackers > 0 {
        pop_lsb!(from, promo_attackers);
        attacks = PATT[c][from as usize] & opps;
        while attacks > 0 {
            pop_lsb!(cidx, attacks);
            f = from << 6;
            push_move!(moves, QPROMO_CAP | cidx | f);
            push_move!(moves, PROMO_CAP  | cidx | f);
            push_move!(moves, BPROMO_CAP | cidx | f);
            push_move!(moves, RPROMO_CAP | cidx | f);
        }
    }
}

fn en_passants(moves: &mut MoveList, pawns: u64, sq: u16, c: usize) {
    let mut attackers: u64 = PATT[c ^ 1][sq as usize] & pawns;
    let mut cidx: u16;
    while attackers > 0 {
        pop_lsb!(cidx, attackers);
        push_move!(moves, ENP | sq | cidx << 6 );
    }
}

#[inline(always)]
fn shift(bb: u64, c: usize) -> u64 {
    if c == WH {bb >> 8} else {bb << 8}
}

#[inline(always)]
fn idx_shift<const AMOUNT: u16>(idx: u16, c: usize) -> u16 {
    if c == WH {idx + AMOUNT} else {idx - AMOUNT}
}

fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occupied: u64, pawns: u64) {
    let empty: u64 = !occupied;
    let mut pushable_pawns: u64 = shift(empty, SIDE) & pawns;
    let mut dbl_pushable_pawns: u64 = shift(shift(empty & DBLRANK[SIDE], SIDE) & empty, SIDE) & pawns;
    let mut promotable_pawns: u64 = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    let mut idx: u16;
    while pushable_pawns > 0 {
        pop_lsb!(idx, pushable_pawns);
        push_move!(moves, idx_shift::<8>(idx, SIDE) | idx << 6);
    }
    while promotable_pawns > 0 {
        pop_lsb!(idx, promotable_pawns);
        let to: u16 = idx_shift::<8>(idx, SIDE);
        let f: u16 = idx << 6;
        push_move!(moves, QPROMO | to | f);
        push_move!(moves, PROMO  | to | f);
        push_move!(moves, BPROMO | to | f);
        push_move!(moves, RPROMO | to | f);
    }
    while dbl_pushable_pawns > 0 {
        pop_lsb!(idx, dbl_pushable_pawns);
        push_move!(moves, DBL | idx_shift::<16>(idx, SIDE) | idx << 6);
    }
}
