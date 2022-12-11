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

#[derive(Copy, Clone, Default)]
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

impl MoveList {
    #[inline(always)]
    pub fn push(&mut self, m: u16) {
        self.list[self.len] = m;
        self.len += 1;
    }
}