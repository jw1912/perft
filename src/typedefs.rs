#[derive(Copy, Clone)]
pub struct Pos {
    pub pc: [u64; 6],
    pub s: [u64; 2],
    pub c: usize,
    pub state: State,
}

#[derive(Copy, Clone)]
pub struct State {
    pub enp: u8,
    pub hfm: u8,
    pub cr: u8,
}

#[derive(Copy, Clone)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flag: u8,
    pub mpc: u8,
}

pub struct MoveList {
    pub list: [Move; 252],
    pub len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {list: unsafe {#[allow(clippy::uninit_assumed_init)] std::mem::MaybeUninit::uninit().assume_init()}, len: 0}
    }
}
impl MoveList {
    #[inline(always)]
    pub fn push(&mut self, from: u8, to: u8, flag: u8, mpc: u8) {
        self.list[self.len] = Move {from, to, flag, mpc};
        self.len += 1;
    }
}

#[derive(Clone, Copy)]
pub struct Mask {
    pub bitmask: u64,
    pub diag: u64,
    pub antidiag: u64,
    pub file: u64,
}
