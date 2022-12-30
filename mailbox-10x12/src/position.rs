use crate::consts::*;

#[derive(Clone, Copy)]
pub struct Position {
    pub board: [u8; 120],
    pub c: bool,
    pub enp: u16,
    pub cr: u8,
    pub kings: [u8; 2],
}


#[inline]
pub fn colour(pc: u8) -> u8 {
    pc & 8
}

#[inline]
pub fn piece(pc: u8) -> u8 {
    pc & 7
}

impl Position {
    /// Gets the piece in a square on the actual 8x8 board indexed by 0..=63.
    #[inline(always)]
    pub fn get_square(&self, idx: u16) -> u8 {
        self.board[usize::from(MAILBOX_64[usize::from(idx)])]
    }

    /// Sets the value in a square on the actual 8x8 board indexed by 0..=63.
    #[inline(always)]
    pub fn set_square(&mut self, idx: u16, val: u8) {
        self.board[usize::from(MAILBOX_64[usize::from(idx)])] = val;
    }

    pub fn is_square_attacked(&self, sq_64: u8, side: usize) -> bool {
        let sq_120 = usize::from(MAILBOX_64[sq_64 as usize]);
        let opp = ((side ^ 1) as u8) << 3;
        for att in PAWN_CAPS[side] {
            let pc = self.board[((sq_120 as i16) + att) as usize];
            if colour(pc) == opp && piece(pc) == P {return true}
        }
        for i in 0..8 {
            let att = OFFSETS[N as usize][i];
            let pc = self.board[((sq_120 as i16) + att) as usize];
            if colour(pc) == opp && piece(pc) == N {return true}
        }
        for i in 0..8 {
            let att = OFFSETS[K as usize][i];
            let pc = self.board[((sq_120 as i16) + att) as usize];
            if colour(pc) == opp && piece(pc) == K {return true}
        }
        for i in 0..4 {
            let dir = OFFSETS[B as usize][i];
            let mut to: i16 = dir + sq_120 as i16;
            'dir: loop {
                let target: u8 = self.board[to as usize];
                if target != E {
                    if colour(target) == opp  && (piece(target) == Q || piece(target) == B) {return true}
                    break 'dir
                }
                to += dir;
            }
        }
        for i in 0..4 {
            let dir = OFFSETS[R as usize][i];
            let mut to: i16 = dir + sq_120 as i16;
            'dir: loop {
                let target: u8 = self.board[to as usize];
                if target != E {
                    if colour(target) == opp  && (piece(target) == Q || piece(target) == R) {return true}
                    break 'dir
                }
                to += dir;
            }
        }
        false
    }

    pub fn do_move(&mut self, m: u16) -> bool {
        // getting move info
        let from: u16 = (m >> 6) & 63;
        let to: u16 = m & 63;
        let flag: u16 = m >> 12;
        let mpc: u8 = self.get_square(from);
        let side: usize = usize::from(self.c);

        // updating board
        self.c = !self.c;
        self.enp = 0;
        self.cr &= CR[usize::from(to)] & CR[usize::from(from)];
        self.set_square(from, E);
        self.set_square(to, mpc);
        if piece(mpc) == K {self.kings[side] = to as u8}
        match flag {
            QUIET => {},
            DBL => self.enp = if side == WH {to - 8} else {to + 8},
            KS => {
                let (idx1, idx2): (u16, u16) = CKM[side];
                self.set_square(idx1, E);
                self.set_square(idx2, R);
            },
            QS => {
                let (idx1, idx2): (u16, u16) = CQM[side];
                self.set_square(idx1, E);
                self.set_square(idx2, R);
            },
            ENP => self.set_square(to + [8u16.wrapping_neg(), 8u16][side], E),
            PROMO.. => self.set_square(to, (flag as u8 - 1) & 3),
        }

        self.is_square_attacked(self.kings[side], side)
    }
}