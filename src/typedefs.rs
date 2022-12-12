#[derive(Default)]
pub struct Position {
    pub piece: [u64; 6],
    pub side: [u64; 2],
    pub mover: usize,
    pub state: State,
}

#[derive(Copy, Clone, Default)]
pub struct State {
    pub enp: u16,
    pub halfm: u8,
    pub rights: u8,
}

#[derive(Copy, Clone)]
pub struct MoveState {
    pub state: State,
    pub m: u16,
    pub mpc: u8,
    pub cpc: u8,
}

pub struct MoveList {
    pub list: [u16; 252],
    pub len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {list: unsafe {#[allow(clippy::uninit_assumed_init)] std::mem::MaybeUninit::uninit().assume_init()}, len: 0} 
    }
}

#[derive(Clone, Copy)]
pub struct Mask {
    pub bitmask: u64,
    pub diag: u64,
    pub antidiag: u64,
    pub file: u64,
}
