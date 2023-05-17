use super::{
    attacks::Attacks,
    consts::{CASTLE_MASK, Flag, Piece, ROOK_MOVES},
};

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub bb: [u64; 8],
    pub c: bool,
    pub enp_sq: u8,
    pub rights: u8,
}

#[derive(Copy, Clone, Default)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flag: u8,
    pub moved: u8,
}

#[inline]
fn enp_sq(side: usize, sq: u8) -> u8 {
    sq.wrapping_add([8u8.wrapping_neg(), 8u8][side])
}

impl Position {
    #[inline]
    pub fn toggle(&mut self, c: usize, pc: usize, bit: u64) {
        self.bb[pc] ^= bit;
        self.bb[ c] ^= bit;
    }

    #[must_use]
    #[inline]
    pub fn is_sq_att(&self, sq: usize, side: usize, occ: u64) -> bool {
        ( (Attacks::KNIGHT[sq] & self.bb[Piece::KNIGHT])
        | (Attacks::KING  [sq] & self.bb[Piece::KING  ])
        | (Attacks::PAWN  [side][sq] & self.bb[Piece::PAWN  ])
        | (Attacks::rook  (sq, occ) & (self.bb[Piece::ROOK  ] | self.bb[Piece::QUEEN]))
        | (Attacks::bishop(sq, occ) & (self.bb[Piece::BISHOP] | self.bb[Piece::QUEEN]))
        ) & self.bb[side ^ 1] > 0
    }

    #[must_use]
    #[inline]
    pub fn get_pc(&self, bit: u64) -> usize {
        usize::from(
            (self.bb[Piece::KNIGHT] | self.bb[Piece::ROOK]) & bit > 0
        ) | (2 * usize::from(
            (self.bb[Piece::KNIGHT] | self.bb[Piece::PAWN] | self.bb[Piece::QUEEN]) & bit > 0)
        ) | (4 * usize::from(
            (self.bb[Piece::BISHOP] | self.bb[Piece::ROOK] | self.bb[Piece::QUEEN]) & bit > 0)
        )
    }

    pub fn make(&mut self, mov: Move) -> bool {
        // extracting move info
        let side = usize::from(self.c);
        let bb_from = 1 << mov.from;
        let bb_to = 1 << mov.to;
        let captured = if mov.flag & Flag::CAP == 0 {
            Piece::EMPTY
        } else {
            self.get_pc(bb_to)
        };

        // updating state
        self.c = !self.c;
        self.enp_sq = 0;
        self.rights &= CASTLE_MASK[usize::from(mov.to)] & CASTLE_MASK[usize::from(mov.from)];

        // move piece
        self.toggle(side, usize::from(mov.moved), bb_from | bb_to);

        // captures
        if captured != Piece::EMPTY { self.toggle(side ^ 1, captured, bb_to) }

        // more complex moves
        match mov.flag {
            Flag::DBL => self.enp_sq = enp_sq(side, mov.to),
            Flag::KS | Flag::QS => {
                let bits = ROOK_MOVES[usize::from(mov.flag == Flag::KS)][side];
                self.toggle(side, Piece::ROOK, bits);
            },
            Flag::ENP => {
                let bits = 1 << enp_sq(side, mov.to);
                self.toggle(side ^ 1, Piece::PAWN, bits);
            },
            Flag::NPR.. => {
                self.bb[Piece::PAWN] ^= bb_to;
                self.bb[usize::from((mov.flag & 3) + 3)] ^= bb_to;
            }
            _ => {}
        }

        // is move legal?
        let king_sq = (self.bb[Piece::KING] & self.bb[side]).trailing_zeros();
        self.is_sq_att(king_sq as usize, side, self.bb[0] | self.bb[1])
    }
}
