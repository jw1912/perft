use super::{
    attacks::*,
    consts::*,
};

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub  bb: [u64; 8],
    pub   c: bool,
    pub enp: u8,
    pub  cr: u8,
}

#[derive(Copy, Clone, Default)]
pub struct Move {
    pub from: u8,
    pub   to: u8,
    pub flag: u8,
    pub  mpc: u8,
}

impl Position {
    #[inline(always)]
    pub fn toggle(&mut self, c: usize, pc: usize, bit: u64) {
        self.bb[pc] ^= bit;
        self.bb [c] ^= bit;
    }

    #[inline(always)]
    pub fn is_sq_att(&self, sq: usize, side: usize, occ: u64) -> bool {
        ( (Attacks::KNIGHT[sq] & self.bb[Piece::KNIGHT])
        | (Attacks::KING  [sq] & self.bb[Piece::KING  ])
        | (Attacks::PAWN  [side][sq] & self.bb[Piece::PAWN  ])
        | (Attacks::rook  (sq, occ) & (self.bb[Piece::ROOK  ] | self.bb[Piece::QUEEN]))
        | (Attacks::bishop(sq, occ) & (self.bb[Piece::BISHOP] | self.bb[Piece::QUEEN]))
        ) & self.bb[side ^ 1] > 0
    }

    #[inline(always)]
    pub fn get_pc(&self, bit: u64) -> usize {
        usize::from(
            (self.bb[Piece::KNIGHT] | self.bb[Piece::ROOK]) & bit > 0
        ) | (2 * usize::from(
            (self.bb[Piece::KNIGHT] | self.bb[Piece::PAWN] | self.bb[Piece::QUEEN]) & bit > 0)
        ) | (4 * usize::from(
            (self.bb[Piece::BISHOP] | self.bb[Piece::ROOK] | self.bb[Piece::QUEEN]) & bit > 0)
        )
    }

    pub fn make(&mut self, m: Move) -> bool {
        // extracting move info
        let f = 1 << m.from;
        let t = 1 << m.to;
        let cpc = if m.flag & Flag::CAP == 0 { Piece::EMPTY } else { self.get_pc(t) };
        let side = usize::from(self.c);

        // updating state
        self.c = !self.c;
        self.enp = 0;
        self.cr &= CR[usize::from(m.to)] & CR[usize::from(m.from)];

        // move piece
        self.toggle(side, usize::from(m.mpc), f | t);

        // captures
        if cpc != Piece::EMPTY { self.toggle(side ^ 1, cpc, t) }

        // more complex moves
        match m.flag {
            Flag::DBL => self.enp = if side == Side::WHITE { m.to - 8 } else { m.to + 8 },
            Flag::KS | Flag::QS => {
                let bits = CM[usize::from(m.flag == Flag::KS)][side];
                self.toggle(side, Piece::ROOK, bits);
            },
            Flag::ENP => {
                let bits = 1 << (m.to.wrapping_add([8u8.wrapping_neg(), 8u8][side]));
                self.toggle(side ^ 1, Piece::PAWN, bits);
            },
            Flag::NPR.. => {
                self.bb[Piece::PAWN] ^= t;
                self.bb[usize::from((m.flag & 3) + 3)] ^= t;
            }
            _ => {}
        }

        // is move legal?
        let king_sq = (self.bb[Piece::KING] & self.bb[side]).trailing_zeros();
        self.is_sq_att(king_sq as usize, side, self.bb[0] | self.bb[1])
    }
}
