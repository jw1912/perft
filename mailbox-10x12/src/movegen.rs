use crate::{position::{Position, colour, piece}, consts::*};

pub struct MoveList {
    pub list: [u16; 252],
    pub len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {list: [0; 252], len: 0}
    }
}
impl MoveList {
    #[inline(always)]
    fn push(&mut self, m: u16) {
        self.list[self.len] = m;
        self.len += 1;
    }
}

impl Position {
    pub fn gen(&self, moves: &mut MoveList) {
        let side: u8 = u8::from(self.c) << 3;
        for (sq, &sq_120) in MAILBOX_64.iter().enumerate() {
            let pc: u8 = self.board[sq_120 as usize];
            if pc == E {continue}
            else if colour(pc) == side {
                let f: u16 = (sq as u16) << 6;
                let pc_type: u8 = piece(pc);
                if pc_type == P {
                    let rank: u8 = (sq as u8) >> 3;
                    if rank == PROMO_RANKS[usize::from(self.c)] {self.gen_promos(moves, sq_120, side, f)}
                    else {self.gen_pawns(moves, sq_120, side, f, rank == DOUBLE_RANKS[usize::from(self.c)])}
                } else {self.gen_piece(moves, sq_120, pc_type, side, f)}
            }
        }
        let s = usize::from(self.c);
        if self.cr & CS[s] > 0 && !self.is_square_attacked(4 + 56 * self.c as u8, s) {self.castles(moves)}
        if self.enp > 0 {self.en_passants(moves, s)}
    }

    fn gen_piece(&self, moves: &mut MoveList, sq_120: u8, pc_type: u8, side: u8, f: u16) {
        for dir in OFFSETS[pc_type as usize] {
            // reached the end of possible directions to move in
            if dir == 0 {break}
            let mut to: i16 = dir + sq_120 as i16;
            'dir: loop {
                let target: u8 = self.board[to as usize];
                // off the board
                if target == XX {break 'dir}
                let m: u16 = f | MAILBOX_120[to as usize] as u16;
                // non-empty square
                if target != E {
                    // capture
                    if colour(target) != side {moves.push(m)}
                    break 'dir
                }
                moves.push(m);
                // knights & kings can only move once in any direction
                if NON_SLIDER[pc_type as usize] {break 'dir}
                to += dir;
            }
        }
    }

    fn gen_pawns(&self, moves: &mut MoveList, sq_120: u8, side: u8, f: u16, doubles: bool) {
        let stm: usize = usize::from(self.c);
        for attack in PAWN_CAPS[stm] {
            let to: i16 = sq_120 as i16 + attack;
            let target: u8 = self.board[to as usize];
            if target == XX || target == E {continue}
            else if colour(target) != side {moves.push(f | MAILBOX_120[to as usize] as u16)}
        }
        let mut to: i16 = sq_120 as i16 + PUSH[stm];
        if self.board[to as usize] == E {
            moves.push(f | MAILBOX_120[to as usize] as u16);
            if doubles {
                to += PUSH[stm];
                if self.board[to as usize] == E {
                    moves.push((DBL << 12) | f | MAILBOX_120[to as usize] as u16)
                }
            }
        }
    }

    fn gen_promos(&self, moves: &mut MoveList, sq_120: u8, side: u8, f: u16) {
        let stm: usize = usize::from(self.c);
        for attack in PAWN_CAPS[stm] {
            let to: usize = (sq_120 as i16 + attack) as usize;
            let target: u8 = self.board[to];
            if target == XX || target == E {continue}
            else if colour(target) != side {
                let m: u16 = f | MAILBOX_120[to] as u16;
                moves.push(m | QPROMO << 12);
                moves.push(m | PROMO  << 12);
                moves.push(m | BPROMO << 12);
                moves.push(m | RPROMO << 12);
            }
        }
        let to: usize = (sq_120 as i16 + PUSH[stm]) as usize;
        if self.board[to] == E {
            let m: u16 = f | MAILBOX_120[to] as u16;
            moves.push(m | QPROMO << 12);
            moves.push(m | PROMO  << 12);
            moves.push(m | BPROMO << 12);
            moves.push(m | RPROMO << 12);
        }
    }

    fn castles(&self, moves: &mut MoveList) {
        let r: u8 = self.cr;
        if self.c {
            if r & BQS > 0 && self.board[92] == E && self.board[93] == E && self.board[94] == E && !self.is_square_attacked(59, BL) {moves.push(60 << 6 | 58 | QS << 12)}
            if r & BKS > 0 && self.board[96] == E && self.board[97] == E && !self.is_square_attacked(61, BL) {moves.push(60 << 6 | 62 | KS << 12)}
        } else {
            if r & WQS > 0 && self.board[22] == E && self.board[23] == E && self.board[24] == E && !self.is_square_attacked(3, WH) {moves.push(4 << 6 | 2 | QS << 12)}
            if r & WKS > 0 && self.board[26] == E && self.board[27] == E && !self.is_square_attacked(5, WH) {moves.push(4 << 6 | 6 | KS << 12)}
        }
    }

    fn en_passants(&self, moves: &mut MoveList, s: usize) {
        let enp_120 = MAILBOX_64[self.enp as usize];
        for att in PAWN_CAPS[s ^ 1] {
            let from = ((enp_120 as i16) + att) as u16;
            let target = self.board[from as usize];
            if piece(target) == P && colour(target) == (s as u8) << 3 {moves.push((MAILBOX_120[from as usize] as u16) << 6 | self.enp | ENP << 12)}
        }
    }
}