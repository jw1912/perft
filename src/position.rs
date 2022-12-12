use super::*;
use std::hint::unreachable_unchecked;

macro_rules! msb {($x:expr, $t:ty) => {63 ^ $x.leading_zeros() as $t}}

pub fn bishop_attacks(idx: usize, occ: u64) -> u64 {
    let mut ne: u64 = NE[idx];
    let mut sq: usize = lsb!((ne & occ) | MSB, usize);
    ne ^= NE[sq];
    let mut nw: u64 = NW[idx];
    sq = lsb!((nw & occ) | MSB, usize);
    nw ^= NW[sq];
    let mut se: u64 = SE[idx];
    sq = msb!((se & occ) | LSB, usize);
    se ^= SE[sq];
    let mut sw: u64 = SW[idx];
    sq = msb!((sw & occ) | LSB, usize);
    sw ^= SW[sq];
    ne | nw | se | sw
}

pub fn rook_attacks(idx: usize, occ: u64) -> u64 {
    let mut n: u64 = N[idx];
    let mut sq: usize = lsb!((n & occ) | MSB, usize);
    n ^= N[sq];
    let mut e: u64 = E[idx];
    sq = lsb!((e & occ )| MSB, usize);
    e ^= E[sq];
    let mut s: u64 = S[idx];
    sq = msb!((s & occ) | LSB, usize);
    s ^= S[sq];
    let mut w: u64 = W[idx];
    sq = msb!((w & occ) | LSB, usize);
    w ^= W[sq];
    n | e | s | w
}

#[inline(always)]
pub fn is_square_attacked(idx: usize, side: usize, occ: u64) -> bool {
    unsafe {
    let other: usize = side ^ 1;
    let s: u64 = POS.side[other];
    let opp_queen: u64 = POS.piece[QUEEN] & s;
    (NATT[idx] & POS.piece[KNIGHT] & s > 0)
    || (KATT[idx] & POS.piece[KING] & s > 0)
    || (PATT[side][idx] & POS.piece[PAWN] & s > 0)
    || (rook_attacks(idx, occ) & (POS.piece[ROOK] & s | opp_queen) > 0)
    || (bishop_attacks(idx, occ) & (POS.piece[BISHOP] & s | opp_queen) > 0)
    }
}

#[inline(always)]
pub fn is_in_check() -> bool {
    unsafe {
    let king_idx: usize = lsb!(POS.piece[KING] & POS.side[POS.mover], usize);
    is_square_attacked(king_idx, POS.mover, POS.side[0] | POS.side[1])
    }
}

#[inline(always)]
unsafe fn get_piece(bit: u64) -> usize {
    (POS.piece[KNIGHT] & bit > 0) as usize
    + BISHOP * (POS.piece[BISHOP] & bit > 0) as usize
    + ROOK * (POS.piece[ROOK] & bit > 0) as usize
    + QUEEN * (POS.piece[QUEEN] & bit > 0) as usize
    + KING * (POS.piece[KING] & bit > 0) as usize
    + EMPTY * (!(POS.side[0] | POS.side[1]) & bit > 0) as usize
} 

pub fn do_move(m: u16) -> bool {
    unsafe {
    let opp: usize = POS.mover ^ 1;

    // move data
    let (from, to): (usize, usize) = (from!(m), to!(m));
    let (f, t): (u64, u64) = (bit!(from), bit!(to));
    let (mpc, cpc): (usize, usize) = (get_piece(f), get_piece(t));
    let flag: u16 = m & 0xF000;

    // initial updates
    STACK[STACK_IDX] = MoveState { state: POS.state, m, mpc: mpc as u8, cpc: cpc as u8};
    STACK_IDX += 1;
    let mov: u64 = f | t;
    toggle!(POS.mover, mpc, mov);
    POS.state.enp = 0;

    // captures
    if cpc != EMPTY { toggle!(opp, cpc, t); }
    if cpc == ROOK { POS.state.rights &= CASTLE_RIGHTS[to]; }

    // piece-specific updates
    match mpc {
        PAWN => {
            if flag == ENP {
                let p: u64 = match opp { WHITE => t << 8, BLACK => t >> 8, _ => unreachable_unchecked() };
                toggle!(opp, PAWN, p);
            } else if flag == DBL {
                POS.state.enp = match POS.mover {WHITE => to - 8, BLACK => to + 8, _ => unreachable_unchecked()} as u16;
            } else if flag >= PROMO {
                let ppc: usize = (((flag >> 12) & 3) + 1) as usize;
                POS.piece[mpc] ^= t;
                POS.piece[ppc] ^= t;
            }
        }
        KING => {
            POS.state.rights &= CASTLE_RIGHTS[from];
            if flag == KS || flag == QS {
                let c: u64 = CASTLE_MOVES[POS.mover][(flag == KS) as usize];
                toggle!(POS.mover, ROOK, c);
            }
        }
        ROOK => POS.state.rights &= CASTLE_RIGHTS[from],
        _ => {}
    }

    // final updates
    POS.state.halfm = (mpc > PAWN && flag != CAP) as u8 * (POS.state.halfm + 1);
    POS.mover ^= 1;

    // is legal?
    let king_idx: usize = lsb!(POS.piece[KING] & POS.side[opp ^ 1], usize);
    let invalid: bool = is_square_attacked(king_idx, opp ^ 1, POS.side[0] | POS.side[1]);
    if invalid { undo_move() }
    invalid
    }
}

pub fn undo_move() {
    unsafe {
    let opp: usize = POS.mover;
    POS.mover ^= 1;

    // restore state
    STACK_IDX -= 1;
    let state: MoveState = STACK[STACK_IDX];

    // move data
    let (mpc, cpc): (usize, usize) = (state.mpc as usize, state.cpc as usize);
    let (from, to): (usize, usize) = (from!(state.m), to!(state.m));
    let (f, t): (u64, u64) = (bit!(from), bit!(to));
    let flag: u16 = state.m & 0xF000;

    // initial updates
    POS.state = state.state;
    toggle!(POS.mover, mpc, f | t);

    // captures
    if cpc != EMPTY { toggle!(opp, cpc, t); }

    // piece-specific updates
    match mpc as usize {
        PAWN =>  {
            if flag == ENP {
                let p: u64 = match opp { WHITE => t << 8, BLACK => t >> 8, _ => unreachable_unchecked() };
                toggle!(opp, PAWN, p);
            } else if flag >= PROMO {
                let promo_pc: u16 = ((flag >> 12) & 3) + 1;
                POS.piece[mpc] ^= t;
                POS.piece[promo_pc as usize] ^= t;
            }
        }
        KING => {
            if flag == KS || flag == QS {
                let c: u64 = CASTLE_MOVES[POS.mover][(flag == KS) as usize];
                toggle!(POS.mover, ROOK, c);
            }
        }
        _ => {}
    }
    }
}
