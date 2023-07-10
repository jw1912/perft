use super::{
    attacks::Attacks,
    consts::{Flag, Piece, Right, Side, CASTLE_MASK, ROOK_MOVES},
};

#[derive(Copy, Clone, Default)]
pub struct Position {
    bb: [u64; 8],
    stm: bool,
    enp_sq: u8,
    rights: u8,
}

#[derive(Copy, Clone, Default)]
pub struct Move {
    from: u8,
    to: u8,
    flag: u8,
    moved: u8,
}

impl Move {
    #[must_use]
    pub fn new(from: u8, to: u8, flag: u8, moved: u8) -> Self {
        Self {
            from,
            to,
            flag,
            moved,
        }
    }

    #[must_use]
    pub fn to_uci(self) -> String {
        let idx_to_sq = |i| format!("{}{}", ((i & 7) + b'a') as char, (i / 8) + 1);
        let promo = if self.flag & 0b1000 > 0 {
            ["n", "b", "r", "q"][(self.flag & 0b11) as usize]
        } else {
            ""
        };
        format!("{}{}{}", idx_to_sq(self.from), idx_to_sq(self.to), promo)
    }
}

impl Position {
    // ACCESSOR METHODS

    #[must_use]
    pub fn piece(&self, piece: usize) -> u64 {
        self.bb[piece]
    }

    #[must_use]
    pub fn stm(&self) -> usize {
        usize::from(self.stm)
    }

    #[must_use]
    pub fn rights(&self) -> u8 {
        self.rights
    }

    #[must_use]
    pub fn enp_sq(&self) -> u8 {
        self.enp_sq
    }

    // POSITION INFO

    #[must_use]
    pub fn occ(&self) -> u64 {
        self.bb[Side::WHITE] | self.bb[Side::BLACK]
    }

    #[must_use]
    pub fn king_index(&self) -> usize {
        (self.bb[Piece::KING] & self.bb[usize::from(self.stm)]).trailing_zeros() as usize
    }

    #[must_use]
    pub fn boys(&self) -> u64 {
        self.bb[usize::from(self.stm)]
    }

    #[must_use]
    pub fn opps(&self) -> u64 {
        self.bb[usize::from(!self.stm)]
    }

    #[must_use]
    pub fn attackers_to_square(&self, sq: usize, side: usize, occ: u64) -> u64 {
        ((Attacks::knight(sq) & self.bb[Piece::KNIGHT])
            | (Attacks::king(sq) & self.bb[Piece::KING])
            | (Attacks::pawn(sq, side) & self.bb[Piece::PAWN])
            | (Attacks::rook(sq, occ) & (self.bb[Piece::ROOK] ^ self.bb[Piece::QUEEN]))
            | (Attacks::bishop(sq, occ) & (self.bb[Piece::BISHOP] ^ self.bb[Piece::QUEEN])))
            & self.bb[side ^ 1]
    }

    #[must_use]
    pub fn is_square_attacked(&self, sq: usize, side: usize, occ: u64) -> bool {
        self.attackers_to_square(sq, side, occ) > 0
    }

    #[must_use]
    pub fn get_pc(&self, bit: u64) -> usize {
        for pc in Piece::PAWN..=Piece::QUEEN {
            if bit & self.bb[pc] > 0 {
                return pc;
            }
        }
        0
    }

    // MODIFY POSITION

    pub fn toggle(&mut self, side: usize, piece: usize, bit: u64) {
        self.bb[piece] ^= bit;
        self.bb[side] ^= bit;
    }

    pub fn make(&mut self, mov: Move) {
        // extracting move info
        let side = usize::from(self.stm);
        let bb_from = 1 << mov.from;
        let bb_to = 1 << mov.to;
        let captured = if mov.flag & Flag::CAP == 0 {
            Piece::EMPTY
        } else {
            self.get_pc(bb_to)
        };

        // updating state
        self.stm = !self.stm;
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
            }
            Flag::ENP => {
                let bits = 1 << (mov.to ^ 8);
                self.toggle(side ^ 1, Piece::PAWN, bits);
            }
            Flag::NPR.. => {
                let promo = usize::from((mov.flag & 3) + 3);
                self.bb[Piece::PAWN] ^= bb_to;
                self.bb[promo] ^= bb_to;
            }
            _ => {}
        }
    }

    // CREATE POSITION

    #[must_use]
    pub fn parse_fen(fen: &str) -> Self {
        let mut pos = Self::default();
        let vec: Vec<&str> = fen.split_whitespace().collect();
        let p: Vec<char> = vec[0].chars().collect();

        // board
        let (mut row, mut col) = (7, 0);
        for ch in p {
            if ch == '/' {
                row -= 1;
                col = 0;
            } else if ('1'..='8').contains(&ch) {
                col += ch.to_string().parse::<i16>().unwrap_or(0);
            } else {
                let idx: usize = "PNBRQKpnbrqk"
                    .chars()
                    .position(|element| element == ch)
                    .unwrap_or(6);
                let colour = usize::from(idx > 5);
                pos.toggle(colour, idx + 2 - 6 * colour, 1 << (8 * row + col));
                col += 1;
            }
        }

        // side to move
        pos.stm = vec[1] == "b";

        // castle rights
        for ch in vec[2].chars() {
            pos.rights |= match ch {
                'Q' => Right::WQS,
                'K' => Right::WKS,
                'q' => Right::BQS,
                'k' => Right::BKS,
                _ => 0,
            }
        }

        // en passant square
        pos.enp_sq = if vec[3] == "-" {
            0
        } else {
            let chs: Vec<char> = vec[3].chars().collect();
            8 * chs[1].to_string().parse::<u8>().unwrap_or(0) + chs[0] as u8 - 105
        };

        pos
    }
}
