use super::*;
use std::hint::unreachable_unchecked;

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

#[inline(always)]
pub fn is_sq_att(idx: usize, side: usize, occ: u64) -> bool {
    unsafe {
    let other: usize = side ^ 1;
    let s: u64 = POS.s[other];
    let opp_queen: u64 = POS.pc[Q] & s;
    (NATT[idx] & POS.pc[N] & s > 0)
    || (KATT[idx] & POS.pc[K] & s > 0)
    || (PATT[side][idx] & POS.pc[P] & s > 0)
    || (ratt(idx, occ) & (POS.pc[R] & s | opp_queen) > 0)
    || (batt(idx, occ) & (POS.pc[B] & s | opp_queen) > 0)
    }
}

#[inline(always)]
pub fn in_check() -> bool {
    unsafe {
    let king_idx: usize = lsb!(POS.pc[K] & POS.s[POS.c], usize);
    is_sq_att(king_idx, POS.c, POS.s[0] | POS.s[1])
    }
}

pub fn do_move(m: u16) -> bool {
    unsafe {
    let (from, to): (usize, usize) = (from!(m), to!(m));
    let (f, t): (u64, u64) = (bit!(from), bit!(to));
    let (mpc, cpc): (usize, usize) = (POS.sq[from] as usize, POS.sq[to] as usize);
    let flag: u16 = m & 0xF000;
    let opp: usize = POS.c ^ 1;

    STACK[STACK_IDX] = MoveState { state: POS.state, m, mpc: mpc as u8, cpc: cpc as u8};
    STACK_IDX += 1;
    let mov: u64 = f | t;
    toggle!(POS.c, mpc, mov);
    POS.sq[from] = E as u8;
    POS.sq[to] = mpc as u8;
    POS.state.enp = 0;
    if cpc != E { toggle!(opp, cpc, t); }
    if cpc == R { POS.state.cr &= CR[to]; }
    match mpc {
        P => {
            if flag == ENP {
                let p_idx: usize = if opp == WH {to + 8} else {to - 8};
                let p: u64 = bit!(p_idx);
                toggle!(opp, P, p);
                POS.sq[p_idx] = E as u8;
            } else if flag == DBL {
                POS.state.enp = match POS.c {WH => to - 8, BL => to + 8, _ => unreachable_unchecked()} as u16;
            } else if flag >= PROMO {
                let ppc: u16 = ((flag >> 12) & 3) + 1;
                POS.pc[mpc] ^= t;
                POS.pc[ppc as usize] ^= t;
                POS.sq[to] = ppc as u8;
            }
        }
        K => {
            POS.state.cr &= CR[from];
            if flag == KS || flag == QS {
                let (c, idx1, idx2): (u64, usize, usize) = CASTLE_MOVES[POS.c][(flag == KS) as usize];
                POS.sq.swap(idx1, idx2);
                toggle!(POS.c, R, c);
            }
        }
        R => POS.state.cr &= CR[from],
        _ => {}
    }
    POS.state.hfm = (mpc > P && flag != CAP) as u8 * (POS.state.hfm + 1);
    POS.c ^= 1;

    let king_idx: usize = lsb!(POS.pc[K] & POS.s[opp ^ 1], usize);
    let invalid: bool = is_sq_att(king_idx, opp ^ 1, POS.s[0] | POS.s[1]);
    if invalid { undo_move() }
    invalid
    }
}

pub fn undo_move() {
    unsafe {
    STACK_IDX -= 1;
    let state: MoveState = STACK[STACK_IDX];
    let (mpc, cpc): (usize, usize) = (state.mpc as usize, state.cpc as usize);
    let (from, to): (usize, usize) = (from!(state.m), to!(state.m));
    let (f, t): (u64, u64) = (bit!(from), bit!(to));
    let flag: u16 = state.m & 0xF000;
    let opp: usize = POS.c;

    POS.c ^= 1;
    POS.state = state.state;
    let mov: u64 = f | t;
    toggle!(POS.c, mpc, mov);
    POS.sq[from] = mpc as u8;
    POS.sq[to] = cpc as u8;
    if cpc != E { toggle!(opp, cpc, t); }
    match mpc as usize {
        P =>  {
            if flag == ENP {
                let p_idx: usize = if opp == WH {to + 8} else {to - 8};
                let p: u64 = bit!(p_idx);
                toggle!(opp, P, p);
                POS.sq[p_idx] = P as u8;
            } else if flag >= PROMO {
                POS.pc[mpc] ^= t;
                POS.pc[(((flag >> 12) & 3) + 1) as usize] ^= t;
            }
        }
        K => {
            if flag == KS || flag == QS {
                let (c, idx1, idx2): (u64, usize, usize) = CASTLE_MOVES[POS.c][(flag == KS) as usize];
                POS.sq.swap(idx1, idx2);
                toggle!(POS.c, R, c);
            }
        }
        _ => {}
    }}
}
