use crate::{position::Position, consts::*};

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
    pub fn push(&mut self, m: u16) {
        self.list[self.len] = m;
        self.len += 1;
    }
}

#[inline]
fn colour(pc: u8) -> u8 {
    pc & 8
}

#[inline]
fn piece(pc: u8) -> u8 {
    pc & 7
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
}