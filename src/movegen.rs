use super::{
    consts::*,
    position::{batt, ratt, Move, Position},
};

macro_rules! pop_lsb {
    ($idx:expr, $x:expr) => {
        $idx = $x.trailing_zeros() as u8;
        $x &= $x - 1
    };
}

pub struct MoveList {
    pub list: [Move; 252],
    pub len: usize,
}

impl MoveList {
    #[inline(always)]
    fn push(&mut self, from: u8, to: u8, flag: u8, mpc: u8) {
        self.list[self.len] = Move { from, to, flag, mpc };
        self.len += 1;
    }

    #[inline(always)]
    fn uninit() -> Self {
        Self {
            list: unsafe {
                #[allow(clippy::uninit_assumed_init, invalid_value)]
                std::mem::MaybeUninit::uninit().assume_init()
            },
            len: 0,
        }
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

impl Position {
    pub fn gen(&self) -> MoveList {
        let mut moves = MoveList::uninit();
        let side = usize::from(self.c);
        let occ = self.bb[0] | self.bb[1];
        let friends = self.bb[side];
        let opps = self.bb[side ^ 1];
        let pawns = self.bb[P] & friends;

        // castling
        if self.cr & CS[side] > 0 && !self.is_sq_att(4 + 56 * (side == BL) as usize, side, occ) {
            self.castles(&mut moves, occ)
        }

        // pawns
        if side == WH {
            pawn_pushes::<WH>(&mut moves, occ, pawns);
        } else {
            pawn_pushes::<BL>(&mut moves, occ, pawns);
        }
        if self.enp > 0 {
            en_passants(&mut moves, pawns, self.enp, side)
        }
        pawn_captures(&mut moves, pawns, opps, side);

        // other pieces
        pc_moves::<N>(&mut moves, occ, opps, friends & self.bb[N]);
        pc_moves::<B>(&mut moves, occ, opps, friends & self.bb[B]);
        pc_moves::<R>(&mut moves, occ, opps, friends & self.bb[R]);
        pc_moves::<Q>(&mut moves, occ, opps, friends & self.bb[Q]);
        pc_moves::<K>(&mut moves, occ, opps, friends & self.bb[K]);

        moves
    }

    #[inline(always)]
    fn castles(&self, moves: &mut MoveList, occ: u64) {
        if self.c {
            if self.cr & BQS > 0 && occ & B8C8D8 == 0 && !self.is_sq_att(59, BL, occ) {
                moves.push(60, 58, QS, K as u8)
            }
            if self.cr & BKS > 0 && occ & F8G8 == 0 && !self.is_sq_att(61, BL, occ) {
                moves.push(60, 62, KS, K as u8)
            }
        } else {
            if self.cr & WQS > 0 && occ & B1C1D1 == 0 && !self.is_sq_att(3, WH, occ) {
                moves.push(4, 2, QS, K as u8)
            }
            if self.cr & WKS > 0 && occ & F1G1 == 0 && !self.is_sq_att(5, WH, occ) {
                moves.push(4, 6, KS, K as u8)
            }
        }
    }
}

#[inline(always)]
fn pc_moves<const PC: usize>(moves: &mut MoveList, occ: u64, opps: u64, mut attackers: u64) {
    let mut from;
    let mut attacks;
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
        encode::<PC, QUIET>(moves, attacks & !occ, from);
    }
}

#[inline(always)]
fn pawn_captures(moves: &mut MoveList, mut attackers: u64, opps: u64, c: usize) {
    let (mut from, mut to, mut attacks);
    let mut promo_attackers = attackers & PENRANK[c];
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
            moves.push(from, to, QPC, P as u8);
            moves.push(from, to, NPC, P as u8);
            moves.push(from, to, BPC, P as u8);
            moves.push(from, to, RPC, P as u8);
        }
    }
}

#[inline(always)]
fn en_passants(moves: &mut MoveList, pawns: u64, sq: u8, c: usize) {
    let mut attackers = PATT[c ^ 1][sq as usize] & pawns;
    let mut from;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        moves.push(from, sq, ENP, P as u8);
    }
}

fn shift<const SIDE: usize>(bb: u64) -> u64 {
    if SIDE == WH {
        bb >> 8
    } else {
        bb << 8
    }
}

fn idx_shift<const SIDE: usize, const AMOUNT: u8>(idx: u8) -> u8 {
    if SIDE == WH {
        idx + AMOUNT
    } else {
        idx - AMOUNT
    }
}

#[inline(always)]
fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occupied: u64, pawns: u64) {
    let empty = !occupied;
    let mut pushable_pawns = shift::<SIDE>(empty) & pawns;
    let mut dbl_pushable_pawns =
        shift::<SIDE>(shift::<SIDE>(empty & DBLRANK[SIDE]) & empty) & pawns;
    let mut promotable_pawns = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    let mut from;

    while pushable_pawns > 0 {
        pop_lsb!(from, pushable_pawns);
        moves.push(from, idx_shift::<SIDE, 8>(from), QUIET, P as u8);
    }

    while promotable_pawns > 0 {
        pop_lsb!(from, promotable_pawns);
        let to = idx_shift::<SIDE, 8>(from);
        moves.push(from, to, QPR, P as u8);
        moves.push(from, to, NPR, P as u8);
        moves.push(from, to, BPR, P as u8);
        moves.push(from, to, RPR, P as u8);
    }

    while dbl_pushable_pawns > 0 {
        pop_lsb!(from, dbl_pushable_pawns);
        moves.push(from, idx_shift::<SIDE, 16>(from), DBL, P as u8);
    }
}
