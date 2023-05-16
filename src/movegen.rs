use super::{
    attacks::Attacks,
    consts::*,
    position::{Move, Position},
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

impl Default for MoveList {
    fn default() -> Self {
        Self { list: [Move::default(); 252], len: 0 }
    }
}

impl MoveList {
    #[inline(always)]
    fn push(&mut self, from: u8, to: u8, flag: u8, mpc: usize) {
        self.list[self.len] = Move { from, to, flag, mpc: mpc as u8 };
        self.len += 1;
    }
}

#[inline(always)]
fn encode<const PC: usize, const FLAG: u8>(moves: &mut MoveList, mut attacks: u64, from: u8) {
    let mut to: u8;
    while attacks > 0 {
        pop_lsb!(to, attacks);
        moves.push(from, to, FLAG, PC);
    }
}

impl Position {
    pub fn gen(&self) -> MoveList {
        let mut moves = MoveList::default();
        let side = usize::from(self.c);

        // reused bitboards
        let occ = self.bb[0] | self.bb[1];
        let boys = self.bb[side];
        let opps = self.bb[side ^ 1];
        let pawns = self.bb[Piece::PAWN] & boys;

        // castling
        if self.cr & Right::SIDE[side] > 0 && !self.is_sq_att(4 + 56 * usize::from(side == Side::BLACK), side, occ) {
            self.castles(&mut moves, occ)
        }

        // pawns
        if side == Side::WHITE {
            pawn_pushes::<{ Side::WHITE }>(&mut moves, occ, pawns);
        } else {
            pawn_pushes::<{ Side::BLACK }>(&mut moves, occ, pawns);
        }
        if self.enp > 0 {
            en_passants(&mut moves, pawns, self.enp, side)
        }
        pawn_captures(&mut moves, pawns, opps, side);

        // other pieces
        pc_moves::<{ Piece::KNIGHT }>(&mut moves, occ, opps, boys & self.bb[Piece::KNIGHT]);
        pc_moves::<{ Piece::BISHOP }>(&mut moves, occ, opps, boys & self.bb[Piece::BISHOP]);
        pc_moves::<{ Piece::ROOK   }>(&mut moves, occ, opps, boys & self.bb[Piece::ROOK  ]);
        pc_moves::<{ Piece::QUEEN  }>(&mut moves, occ, opps, boys & self.bb[Piece::QUEEN ]);
        pc_moves::<{ Piece::KING   }>(&mut moves, occ, opps, boys & self.bb[Piece::KING  ]);

        moves
    }

    fn castles(&self, moves: &mut MoveList, occ: u64) {
        if self.c {
            if self.cr & Right::BQS > 0 && occ & Path::BD8 == 0 && !self.is_sq_att(59, Side::BLACK, occ) {
                moves.push(60, 58, Flag::QS, Piece::KING)
            }
            if self.cr & Right::BKS > 0 && occ & Path::FG8 == 0 && !self.is_sq_att(61, Side::BLACK, occ) {
                moves.push(60, 62, Flag::KS, Piece::KING)
            }
        } else {
            if self.cr & Right::WQS > 0 && occ & Path::BD1 == 0 && !self.is_sq_att( 3, Side::WHITE, occ) {
                moves.push( 4,  2, Flag::QS, Piece::KING)
            }
            if self.cr & Right::WKS > 0 && occ & Path::FG1 == 0 && !self.is_sq_att( 5, Side::WHITE, occ) {
                moves.push( 4,  6, Flag::KS, Piece::KING)
            }
        }
    }
}

fn pc_moves<const PC: usize>(moves: &mut MoveList, occ: u64, opps: u64, mut attackers: u64) {
    let mut from;
    let mut attacks;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = match PC {
            Piece::KNIGHT => Attacks::KNIGHT[usize::from(from)],
            Piece::BISHOP => Attacks::bishop(usize::from(from), occ),
            Piece::ROOK   => Attacks::rook  (usize::from(from), occ),
            Piece::QUEEN  => Attacks::queen (usize::from(from), occ),
            Piece::KING   => Attacks::KING  [usize::from(from)],
            _ => 0,
        };
        encode::<PC, { Flag::CAP   }>(moves, attacks & opps, from);
        encode::<PC, { Flag::QUIET }>(moves, attacks & !occ, from);
    }
}

fn pawn_captures(moves: &mut MoveList, mut attackers: u64, opps: u64, c: usize) {
    let (mut from, mut to, mut attacks);
    let mut promo_attackers = attackers & PENRANK[c];
    attackers &= !PENRANK[c];

    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = Attacks::PAWN[c][usize::from(from)] & opps;
        encode::<{ Piece::PAWN }, { Flag::CAP }>(moves, attacks, from);
    }

    while promo_attackers > 0 {
        pop_lsb!(from, promo_attackers);
        attacks = Attacks::PAWN[c][usize::from(from)] & opps;
        while attacks > 0 {
            pop_lsb!(to, attacks);
            moves.push(from, to, Flag::QPC, Piece::PAWN);
            moves.push(from, to, Flag::NPC, Piece::PAWN);
            moves.push(from, to, Flag::BPC, Piece::PAWN);
            moves.push(from, to, Flag::RPC, Piece::PAWN);
        }
    }
}

fn en_passants(moves: &mut MoveList, pawns: u64, sq: u8, c: usize) {
    let mut attackers = Attacks::PAWN[c ^ 1][usize::from(sq)] & pawns;
    let mut from;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        moves.push(from, sq, Flag::ENP, Piece::PAWN);
    }
}

fn shift<const SIDE: usize>(bb: u64) -> u64 {
    if SIDE == Side::WHITE {
        bb >> 8
    } else {
        bb << 8
    }
}

fn idx_shift<const SIDE: usize, const AMOUNT: u8>(idx: u8) -> u8 {
    if SIDE == Side::WHITE {
        idx + AMOUNT
    } else {
        idx - AMOUNT
    }
}

fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occupied: u64, pawns: u64) {
    let mut from;
    let empty = !occupied;

    let mut pushable_pawns = shift::<SIDE>(empty) & pawns;
    let mut promotable_pawns = pushable_pawns & PENRANK[SIDE];
    pushable_pawns &= !PENRANK[SIDE];
    while pushable_pawns > 0 {
        pop_lsb!(from, pushable_pawns);
        let to = idx_shift::<SIDE, 8>(from);
        moves.push(from, to, Flag::QUIET, Piece::PAWN);
    }

    while promotable_pawns > 0 {
        pop_lsb!(from, promotable_pawns);
        let to = idx_shift::<SIDE, 8>(from);
        moves.push(from, to, Flag::QPR, Piece::PAWN);
        moves.push(from, to, Flag::NPR, Piece::PAWN);
        moves.push(from, to, Flag::BPR, Piece::PAWN);
        moves.push(from, to, Flag::RPR, Piece::PAWN);
    }

    let mut dbl_pushable_pawns =
        shift::<SIDE>(shift::<SIDE>(empty & DBLRANK[SIDE]) & empty) & pawns;
    while dbl_pushable_pawns > 0 {
        pop_lsb!(from, dbl_pushable_pawns);
        moves.push(from, idx_shift::<SIDE, 16>(from), Flag::DBL, Piece::PAWN);
    }
}
