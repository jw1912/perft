use crate::consts::*;

pub struct Position {
    pub board: [u8; 120],
    pub c: bool,
    pub enp: u16,
    pub cr: u8,
}

impl Position {
    /// Gets the piece in a square on the actual board indexed by 0..=63.
    #[inline(always)]
    pub fn get_square(&self, idx: u16) -> u8 {
        self.board[usize::from(MAILBOX_64[usize::from(idx)])]
    }

    /// Sets the value in a square on the actual board indexed by 0..=63.
    #[inline(always)]
    pub fn set_square(&mut self, idx: u16, val: u8) {
        self.board[usize::from(MAILBOX_64[usize::from(idx)])] = val;
    }

    pub fn do_move(&mut self, m: u16) -> bool {
        // getting move info
        let from: u16 = (m >> 6) & 63;
        let to: u16 = m & 63;
        let flag: u16 = m >> 12;
        let mpc: u8 = self.get_square(from);
        let cpc: u8 = self.get_square(to);
        let side: usize = usize::from(self.c);

        // updating board
        self.c = !self.c;
        self.enp = 0;
        self.cr &= CR[usize::from(to)] & CR[usize::from(from)];
        self.set_square(from, E);
        self.set_square(to, mpc);
        match flag {
            QUIET => {},
            DBL => self.enp = if side == WH {to - 8} else {to + 8},
            KS => {},
            QS => {},
            ENP => self.set_square(to + [8u16.wrapping_neg(), 8u16][side], E),
            PROMO.. => {}
        }
        false
    }
}