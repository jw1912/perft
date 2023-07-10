use super::{
    attacks::Attacks,
    consts::{CASTLE_MASK, Flag, Piece, ROOK_MOVES, Side},
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

impl Move {
    #[must_use]
    pub fn to_uci(self) -> String {
        let idx_to_sq = |i| format!("{}{}", ((i & 7) + b'a') as char, (i / 8) + 1);
        let promo = if self.flag & 0b1000 > 0 {["n","b","r","q"][(self.flag & 0b11) as usize]} else {""};
        format!("{}{}{}", idx_to_sq(self.from), idx_to_sq(self.to), promo)
    }
}

impl Position {
    #[must_use]
    #[inline]
    pub fn occ(&self) -> u64 {
        self.bb[Side::WHITE] | self.bb[Side::BLACK]
    }

    #[must_use]
    #[inline]
    pub fn king_index(&self) -> usize {
        (self.bb[Piece::KING] & self.bb[usize::from(self.side)]).trailing_zeros() as usize
    }

    #[must_use]
    #[inline]
    pub fn boys(&self) -> u64 {
        self.bb[usize::from(self.side)]
    }

    #[must_use]
    #[inline]
    pub fn opps(&self) -> u64 {
        self.bb[usize::from(!self.side)]
    }

    #[inline]
    pub fn toggle(&mut self, side: usize, piece: usize, bit: u64) {
        self.bb[piece] ^= bit;
        self.bb[side] ^= bit;
    }

    #[must_use]
    #[inline]
    pub fn attackers_to_square(&self, sq: usize, side: usize, occ: u64) -> u64 {
        ( (Attacks::knight(sq)       & self.bb[Piece::KNIGHT])
        | (Attacks::king  (sq)       & self.bb[Piece::KING  ])
        | (Attacks::pawn  (sq, side) & self.bb[Piece::PAWN  ])
        | (Attacks::rook  (sq, occ) & (self.bb[Piece::ROOK  ] ^ self.bb[Piece::QUEEN]))
        | (Attacks::bishop(sq, occ) & (self.bb[Piece::BISHOP] ^ self.bb[Piece::QUEEN]))
        ) & self.bb[side ^ 1]
    }

    #[must_use]
    #[inline]
    pub fn is_square_attacked(&self, sq: usize, side: usize, occ: u64) -> bool {
        self.attackers_to_square(sq, side, occ) > 0
    }

    #[must_use]
    #[inline]
    pub fn get_pc(&self, bit: u64) -> usize {
        for pc in Piece::PAWN..=Piece::QUEEN {
            if bit & self.bb[pc] > 0 { return pc }
        }
        0
    }

    pub fn make(&mut self, mov: Move) {
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
    }
}
