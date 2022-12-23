use super::{*, position::{ratt, batt}};

macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u8; $x &= $x - 1}}

pub struct ExtPos {occ: u64, pub f: u64, o: u64, p: u64, n: u64, b: u64, r: u64, q: u64, pub k: u64}

impl ExtPos {
    pub fn new(p: &Position, side: usize) -> Self {
        let occ: u64 = p.qbb[1] | p.qbb[2] | p.qbb[3];
        let sides: [u64; 2] = [occ ^ p.qbb[0], p.qbb[0]];
        let odd: u64 = p.qbb[1] ^ p.qbb[2] ^ p.qbb[3];
        Self {
            occ, f: sides[side], o: sides[side ^ 1],
            p: odd & p.qbb[3], n: odd & p.qbb[2], b: p.qbb[2] & p.qbb[3],
            r: odd & p.qbb[1], q: p.qbb[1] & p.qbb[3], k: p.qbb[1] & p.qbb[2],
        }
    }
}

#[inline(always)]
pub fn is_sq_att(idx: usize, side: usize, p: &ExtPos) -> bool {
    (NATT[idx] & p.n & p.o > 0) || (KATT[idx] & p.k & p.o > 0) || (PATT[side][idx] & p.p & p.o > 0)
    || (ratt(idx, p.occ) & (p.r | p.q) & p.o > 0) || (batt(idx, p.occ) & (p.b | p.q) & p.o > 0)
}

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
fn encode<const PC: u8>(moves: &mut MoveList, mut attacks: u64, from: u8) {
    let mut to: u8;
    while attacks > 0 {
        pop_lsb!(to, attacks);
        moves.push(from, to, 0, PC);
    }
}

impl Position {
    pub fn gen(&self, moves: &mut MoveList) {
        // extracting qbb info
        let side: usize = usize::from(self.c);
        let p: ExtPos = ExtPos::new(self, side);
        let pawns: u64 = p.f & p.p;
        // movegen
        if self.cr & CS[side] > 0 && !is_sq_att(4 + 56 * usize::from(side == BL), side, &p) {
            self.castles(moves, &p)
        }
        if side == WH {pawn_pushes::<WH>(moves, p.occ, pawns);} else {pawn_pushes::<BL>(moves, p.occ, pawns);}
        if self.enp > 0 {en_passants(moves, pawns, self.enp, side)}
        pawn_captures(moves, pawns, p.o, side);
        pc_moves::<N>(moves, p.occ, p.f, p.n & p.f);
        pc_moves::<B>(moves, p.occ, p.f, p.b & p.f);
        pc_moves::<R>(moves, p.occ, p.f, p.r & p.f);
        pc_moves::<Q>(moves, p.occ, p.f, p.q & p.f);
        pc_moves::<K>(moves, p.occ, p.f, p.k & p.f);
    }

    #[inline(always)]
    fn castles(&self, moves: &mut MoveList, p: &ExtPos) {
        let cr: u8 = self.cr;
        if self.c {
            if cr & BQS > 0 && p.occ & B8C8D8 == 0 && !is_sq_att(59, BL, p) {moves.push(60, 58, QS, K)}
            if cr & BKS > 0 && p.occ & F8G8 == 0 && !is_sq_att(61, BL, p) {moves.push(60, 62, KS, K)}
        } else {
            if cr & WQS > 0 && p.occ & B1C1D1 == 0 && !is_sq_att(3, WH, p) {moves.push(4, 2, QS, K)}
            if cr & WKS > 0 && p.occ & F1G1 == 0 && !is_sq_att(5, WH, p) {moves.push(4, 6, KS, K)}
        }
    }
}

#[inline(always)]
fn pc_moves<const PC: u8>(moves: &mut MoveList, occ: u64, friends: u64, mut attackers: u64) {
    let mut from: u8;
    let mut attacks: u64;
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
        encode::<PC>(moves, attacks & !friends, from);
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
        encode::<P>(moves, attacks, from);
    }
    while promo_attackers > 0 {
        pop_lsb!(from, promo_attackers);
        attacks = PATT[c][from as usize] & opps;
        while attacks > 0 {
            pop_lsb!(to, attacks);
            moves.push(from, to, QPROMO, P);
            moves.push(from, to, PROMO , P);
            moves.push(from, to, BPROMO, P);
            moves.push(from, to, RPROMO, P);
        }
    }
}

#[inline(always)]
fn en_passants(moves: &mut MoveList, pawns: u64, sq: u8, c: usize) {
    let mut attackers: u64 = PATT[c ^ 1][sq as usize] & pawns;
    let mut from: u8;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        moves.push(from, sq, ENP, P);
    }
}

fn shift<const SIDE: usize>(bb: u64) -> u64 {if SIDE == WH {bb >> 8} else {bb << 8}}
fn idx_shift<const SIDE: usize, const AMOUNT: u8>(idx: u8) -> u8 {if SIDE == WH {idx + AMOUNT} else {idx - AMOUNT}}

#[inline(always)]
fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occupied: u64, pawns: u64) {
    let empty: u64 = !occupied;
    let mut pushable_pawns: u64 = shift::<SIDE>(empty) & pawns;
    let mut dbl_pushable_pawns: u64 = shift::<SIDE>(shift::<SIDE>(empty & DBLRANK[SIDE]) & empty) & pawns;
    let mut promotable_pawns: u64 = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    let mut from: u8;
    while pushable_pawns > 0 {
        pop_lsb!(from, pushable_pawns);
        moves.push(from, idx_shift::<SIDE, 8>(from), QUIET, P);
    }
    while promotable_pawns > 0 {
        pop_lsb!(from, promotable_pawns);
        let to: u8 = idx_shift::<SIDE, 8>(from);
        moves.push(from, to, QPROMO, P);
        moves.push(from, to, PROMO , P);
        moves.push(from, to, BPROMO, P);
        moves.push(from, to, RPROMO, P);
    }
    while dbl_pushable_pawns > 0 {
        pop_lsb!(from, dbl_pushable_pawns);
        moves.push(from, idx_shift::<SIDE, 16>(from), DBL, P);
    }
}
