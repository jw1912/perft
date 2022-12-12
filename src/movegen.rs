use super::{*, position::{is_in_check, is_square_attacked, rook_attacks, bishop_attacks}};
use std::hint::unreachable_unchecked;


macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u16; $x &= $x - 1}}

#[inline(always)]
fn encode_moves(move_list: &mut MoveList, mut attacks: u64, from: u16, flag: u16) {
    let f: u16 = from << 6;
    let mut aidx: u16;
    while attacks > 0 {
        pop_lsb!(aidx, attacks);
        move_list.push(flag | f | aidx);
    }
}

pub fn gen_moves(move_list: &mut MoveList) {
    unsafe {
    let occupied: u64 = POS.side[0] | POS.side[1];
    let friendly: u64 = POS.side[POS.mover];
    let pawns: u64 = POS.piece[PAWN] & POS.side[POS.mover];
    match POS.mover {
        0 => pawn_pushes::<WHITE>(move_list, occupied, pawns),
        1 => pawn_pushes::<BLACK>(move_list, occupied, pawns),
        _ => unreachable_unchecked(),
    }
    if POS.state.rights & SIDES[POS.mover] > 0 && !is_in_check() {castles(move_list, occupied)}
    pawn_captures(move_list, pawns, POS.side[POS.mover ^ 1]);
    if POS.state.enp > 0 {en_passants(move_list, pawns, POS.state.enp)}
    piece_moves::<KNIGHT>(move_list, occupied, friendly);
    piece_moves::<BISHOP>(move_list, occupied, friendly);
    piece_moves::<ROOK  >(move_list, occupied, friendly);
    piece_moves::<QUEEN >(move_list, occupied, friendly);
    piece_moves::<KING  >(move_list, occupied, friendly);
    }
}

unsafe fn piece_moves<const PIECE: usize>(move_list: &mut MoveList, occupied: u64, friendly: u64) {
    let mut from: u16;
    let mut idx: usize;
    let mut attacks: u64;
    let mut attackers: u64 = POS.piece[PIECE] & friendly;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        idx = from as usize;
        attacks = match PIECE {
            KNIGHT => NATT[idx],
            ROOK => rook_attacks(idx, occupied),
            BISHOP => bishop_attacks(idx, occupied),
            QUEEN => rook_attacks(idx, occupied) | bishop_attacks(idx, occupied),
            KING => KATT[idx],
            _ => panic!("Not a valid usize in fn piece_moves_general: {}", PIECE),
        };
        encode_moves(move_list, attacks & POS.side[POS.mover ^ 1], from, CAP);
        encode_moves(move_list, attacks & !occupied, from, QUIET)
    }
}

#[inline(always)]
unsafe fn pawn_captures(move_list: &mut MoveList, mut attackers: u64, opponents: u64) {
    let mut from: u16;
    let mut attacks: u64;
    let mut cidx: u16;
    let mut f: u16;
    let mut promo_attackers: u64 = attackers & PENRANK[POS.mover];
    attackers &= !PENRANK[POS.mover];
    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = PATT[POS.mover][from as usize] & opponents;
        encode_moves(move_list, attacks, from, CAP);
    }
    while promo_attackers > 0 {
        pop_lsb!(from, promo_attackers);
        attacks = PATT[POS.mover][from as usize] & opponents;
        while attacks > 0 {
            pop_lsb!(cidx, attacks);
            f = from << 6;
            move_list.push(QPROMO_CAP  | cidx | f);
            move_list.push(PROMO_CAP | cidx | f);
            move_list.push(BPROMO_CAP | cidx | f);
            move_list.push(RPROMO_CAP   | cidx | f);
        }
    }
}

#[inline(always)]
unsafe fn en_passants(move_list: &mut MoveList, pawns: u64, sq: u16) {
    let mut attackers: u64 = PATT[POS.mover ^ 1][sq as usize] & pawns;
    let mut cidx: u16;
    while attackers > 0 {
        pop_lsb!(cidx, attackers);
        move_list.push( ENP | sq | cidx << 6 );
    }
}

#[inline(always)]
fn shift<const SIDE: usize, const AMOUNT: u8>(bb: u64) -> u64 {
    match SIDE {
        WHITE => bb >> AMOUNT,
        BLACK => bb << AMOUNT,
        _ => panic!("Invalid side in fn shift!"),
    }
}

#[inline(always)]
fn idx_shift<const SIDE: usize, const AMOUNT: u16>(idx: u16) -> u16 {
    match SIDE {
        WHITE => idx + AMOUNT,
        BLACK => idx - AMOUNT,
        _ => panic!("Invalid side in fn shift!"),
    }
}

fn pawn_pushes<const SIDE: usize>(move_list: &mut MoveList, occupied: u64, pawns: u64) {
    let empty: u64 = !occupied;
    let mut pushable_pawns: u64 = shift::<SIDE, 8>(empty) & pawns;
    let mut dbl_pushable_pawns: u64 = shift::<SIDE, 8>(shift::<SIDE, 8>(empty & DBLRANK[SIDE]) & empty) & pawns;
    let mut promotable_pawns: u64 = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    let mut idx: u16;
    while pushable_pawns > 0 {
        pop_lsb!(idx, pushable_pawns);
        move_list.push(idx_shift::<SIDE, 8>(idx) | idx << 6);
    }
    while promotable_pawns > 0 {
        pop_lsb!(idx, promotable_pawns);
        let to: u16 = idx_shift::<SIDE, 8>(idx);
        let f: u16 = idx << 6;
        move_list.push(QPROMO  | to | f);
        move_list.push(PROMO | to | f);
        move_list.push(BPROMO | to | f);
        move_list.push(RPROMO   | to | f);
    }
    while dbl_pushable_pawns > 0 {
        pop_lsb!(idx, dbl_pushable_pawns);
        move_list.push(DBL | idx_shift::<SIDE, 16>(idx) | idx << 6);
    }
}


#[inline(always)]
unsafe fn castles(move_list: &mut MoveList, occupied: u64) {
    match POS.mover {
        WHITE => {
            if POS.state.rights & WQS > 0 && occupied & (B1C1D1) == 0
                && !is_square_attacked(3, WHITE, occupied) {
                move_list.push(QS | 2 | 4 << 6)
            }
            if POS.state.rights & WKS > 0 && occupied & (F1G1) == 0
                && !is_square_attacked(5, WHITE, occupied) {
                move_list.push(KS| 6 | 4 << 6)
            }
        }
        BLACK => {
            if POS.state.rights & BQS > 0 && occupied & (B8C8D8) == 0
                && !is_square_attacked(59, BLACK, occupied) {
                move_list.push(QS | 58 | 60 << 6)
            }
            if POS.state.rights & BKS > 0 && occupied & (F8G8) == 0
                && !is_square_attacked(61, BLACK, occupied) {
                move_list.push(KS | 62 | 60 << 6)
            }
        }
        _ => unreachable_unchecked(),
    }
}
