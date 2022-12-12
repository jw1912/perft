mod typedefs;
mod consts;
mod position;
mod movegen;

pub use typedefs::*;
pub use consts::*;
use movegen::gen_moves;
use position::{do_move, undo_move};
use std::time::Instant;

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

const POSITION: [&str; 6] = [
    STARTPOS, 
    KIWIPETE,
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
];

const EXPECTED: [&[u64]; 6] = [
    &[1, 20, 400, 8_902, 197_281, 4_865_609, 119_060_324],
    &[1, 48, 2_039, 97_862, 4_085_603, 193_690_690],
    &[1, 14, 191, 2_812, 43_238, 674_624, 11_030_083, 178_633_661],
    &[1, 6, 264, 9_467, 422_333, 15_833_292],
    &[1, 44, 1_486, 62_379, 2_103_487, 89_941_194],
    &[1, 46, 2_079, 89_890, 3_894_594, 164_075_551],
];

fn main() {
    unsafe {
    println!("Hello, world!");
    for (i, fen) in POSITION.iter().enumerate() {
        parse_fen(fen);
        println!("Position: {fen}");
        let exp = EXPECTED[i];
        for (d, &exp_count) in exp.iter().enumerate() {
            let now = Instant::now();
            let count: u64 = perft(d as u8);
            assert_eq!(count, exp_count);
            let time = now.elapsed();
            println!("info depth {} time {} nodes {count} Mnps {:.2}", d, time.as_millis(), count as f64 / time.as_micros() as f64);
        }
        println!(" ");
    }
    }
}

fn perft(depth_left: u8) -> u64 {
    if depth_left == 0 { return 1 }
    let mut moves = MoveList::default();
    gen_moves(&mut moves);
    let mut positions: u64 = 0;
    for m_idx in 0..moves.len {
        let m: u16 = moves.list[m_idx];
        if do_move(m) { continue }
        let count: u64 = perft(depth_left - 1);
        positions += count;
        undo_move();
    }
    positions
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