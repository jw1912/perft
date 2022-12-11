mod typedefs;
mod consts;
mod position;

pub use typedefs::*;
pub use consts::*;

static mut POS: Position = Position { piece: [0; 6], side: [0; 2], mover: 0, state: State { enp: 0, halfm: 0, rights: 0 } };
static mut STACK: [MoveState; 128] = [MoveState {state: State { enp: 0, halfm: 0, rights: 0 }, m: 0, mpc: 0, cpc: 0} ; 128];
static mut STACK_IDX: usize = 0;

#[macro_export]
macro_rules! lsb {($x:expr, $t:ty) => {$x.trailing_zeros() as $t}}

#[macro_export]
macro_rules! toggle {
    ($side:expr, $pc:expr, $bit:expr) => {
        POS.piece[$pc] ^= $bit;
        POS.side[$side] ^= $bit;
    };
}

#[macro_export]
macro_rules! bit {($x:expr) => {1 << $x}}

macro_rules! parse {($type: ty, $s: expr, $else: expr) => {$s.parse::<$type>().unwrap_or($else)}}

#[macro_export]
macro_rules! from {($m:expr) => {(($m >> 6) & 63) as usize}}

#[macro_export]
macro_rules! to {($m:expr) => {($m & 63) as usize}}

fn main() {
    unsafe {
    println!("Hello, world!");
    parse_fen(STARTPOS);
    }
}

fn sq_to_idx(sq: &str) -> u16 {
    let chs: Vec<char> = sq.chars().collect();
    8 * parse!(u16, chs[1].to_string(), 0) + chs[0] as u16 - 105
}

unsafe fn parse_fen(fen: &str) {
    POS = Position::default();
    STACK_IDX = 0;
    let vec: Vec<&str> = fen.split_whitespace().collect();
    let p: Vec<char> = vec[0].chars().collect();
    let (mut row, mut col): (i16, i16) = (7, 0);
    for ch in p {
        if ch == '/' { row -= 1; col = 0; }
        else if ('1'..='8').contains(&ch) { col += parse!(i16, ch.to_string(), 0) }
        else {
            let idx: usize = ['P','N','B','R','Q','K','p','n','b','r','q','k'].iter().position(|&element| element == ch).unwrap_or(6);
            toggle!((idx > 5) as usize, idx - 6 * ((idx > 5) as usize), bit!(8 * row + col));
            col += 1;
        }
    }
    POS.mover = (vec[1] == "b") as usize;
    let mut rights: u8 = 0;
    for ch in vec[2].chars() {rights |= match ch {'Q' => WQS, 'K' => WKS, 'q' => BQS, 'k' => BKS, _ => 0}}
    let enp: u16 = if vec[3] == "-" {0} else {sq_to_idx(vec[3])};
    let halfm: u8 = parse!(u8, vec.get(4).unwrap_or(&"0"), 0);
    POS.state = State {enp, halfm, rights};
}