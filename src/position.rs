use super::{
    attacks::Attacks,
    consts::{CASTLE_MASK, Flag, Piece, ROOK_MOVES},
};

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub bb: [u64; 8],
    pub side: bool,
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

impl Position {
    #[inline]
    pub fn toggle(&mut self, side: usize, piece: usize, bit: u64) {
        self.bb[piece] ^= bit;
        self.bb[side] ^= bit;
    }

    #[must_use]
    #[inline]
    pub fn is_sq_att(&self, sq: usize, side: usize, occ: u64) -> bool {
        ( (Attacks::knight(sq)       & self.bb[Piece::KNIGHT])
        | (Attacks::king  (sq)       & self.bb[Piece::KING  ])
        | (Attacks::pawn  (side, sq) & self.bb[Piece::PAWN  ])
        | (Attacks::rook  (sq, occ) & (self.bb[Piece::ROOK  ] ^ self.bb[Piece::QUEEN]))
        | (Attacks::bishop(sq, occ) & (self.bb[Piece::BISHOP] ^ self.bb[Piece::QUEEN]))
        ) & self.bb[side ^ 1] > 0
    }

    #[must_use]
    #[inline]
    pub fn get_pc(&self, bit: u64) -> usize {
        self.bb
            .iter()
            .skip(2) // don't want to consider white/black occupancy bitboards
            .position(|bb| bit & bb > 0) // find piece, if any
            .unwrap_or(usize::MAX - 1) // if no piece, return empty
            .wrapping_add(2) // add back offset due to skip
    }

    pub fn make(&mut self, mov: Move) -> bool {
        // extracting move info
        let side = usize::from(self.side);
        let bb_from = 1 << mov.from;
        let bb_to   = 1 << mov.to;
        let captured = if mov.flag & Flag::CAP == 0 {
            Piece::EMPTY
        } else {
            self.get_pc(bb_to)
        };

        // updating state
        self.side = !self.side;
        self.enp_sq = 0;
        self.rights &= CASTLE_MASK[usize::from(mov.to)] & CASTLE_MASK[usize::from(mov.from)];

        // move piece
        self.toggle(side, usize::from(mov.moved), bb_from ^ bb_to);

        // captures
        if captured != Piece::EMPTY {
            self.toggle(side ^ 1, captured, bb_to);
        }

        // more complex moves
        match mov.flag {
            Flag::DBL => self.enp_sq = mov.to ^ 8,
            Flag::KS | Flag::QS => {
                let bits = ROOK_MOVES[usize::from(mov.flag == Flag::KS)][side];
                self.toggle(side, Piece::ROOK, bits);
            },
            Flag::ENP => {
                let bits = 1 << (mov.to ^ 8);
                self.toggle(side ^ 1, Piece::PAWN, bits);
            },
            Flag::NPR.. => {
                let promo = usize::from((mov.flag & 3) + 3);
                self.bb[Piece::PAWN] ^= bb_to;
                self.bb[promo] ^= bb_to;
            }
            _ => {}
        }

        // is move legal?
        let king_sq = (self.bb[Piece::KING] & self.bb[side]).trailing_zeros();
        self.is_sq_att(king_sq as usize, side, self.bb[0] | self.bb[1])
    }
}
