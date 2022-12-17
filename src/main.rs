mod consts;
mod position;
mod movegen;

pub use position::{Move, Pos};
pub use movegen::MoveList;
pub use consts::*;
use std::time::Instant;

const POSITIONS: [(&str, u8, u64); 6] = [
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119_060_324),
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 7, 178_633_661),
    ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292),
    ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194),
    ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551),
];

fn main() {
    let initial: Instant = Instant::now();
    let mut total: u64 = 0;
    for (fen, d, exp) in POSITIONS {
        let pos = parse_fen(fen);
        println!("Position: {fen}");
        let now: Instant = Instant::now();
        let count: u64 = perft(&pos, d);
        total += count;
        assert_eq!(count, exp);
        let dur = now.elapsed();
        println!("depth {} time {} nodes {count} Mnps {:.2}\n", d, dur.as_millis(), count as f64 / dur.as_micros() as f64);
    }
    let dur = initial.elapsed();
    println!("total time {} nodes {} nps {:.3}", dur.as_millis(), total, total as f64 / dur.as_micros() as f64)
}

fn perft(pos: &Pos, depth_left: u8) -> u64 {
    let mut moves = MoveList::default();
    let mut tmp: Pos;
    let mut positions: u64 = 0;
    pos.gen(&mut moves);
    for m_idx in 0..moves.len {
        tmp = *pos;
        if tmp.do_move(moves.list[m_idx]) { continue }
        positions += if depth_left > 1 {perft(&tmp, depth_left - 1)} else {1};
    }
    positions
}

fn parse_fen(fen: &str) -> Pos {
    let mut pos = Pos { bb: [0; 8], c: 0, enp: 0, cr: 0 };
    let vec: Vec<&str> = fen.split_whitespace().collect();
    let p: Vec<char> = vec[0].chars().collect();
    let (mut row, mut col): (i16, i16) = (7, 0);
    for ch in p {
        if ch == '/' { row -= 1; col = 0; }
        else if ('1'..='8').contains(&ch) { col += ch.to_string().parse::<i16>().unwrap_or(0) }
        else {
            let idx: usize = ['P','N','B','R','Q','K','p','n','b','r','q','k'].iter().position(|&element| element == ch).unwrap_or(6);
            pos.toggle((idx > 5) as usize, idx + 2 - 6 * ((idx > 5) as usize), 1 << (8 * row + col));
            col += 1;
        }
    }
    pos.c = (vec[1] == "b") as u8;
    for ch in vec[2].chars() {pos.cr |= match ch {'Q' => WQS, 'K' => WKS, 'q' => BQS, 'k' => BKS, _ => 0}}
    pos.enp = if vec[3] == "-" {0} else {
        let chs: Vec<char> = vec[3].chars().collect();
        8 * chs[1].to_string().parse::<u8>().unwrap_or(0) + chs[0] as u8 - 105
    };
    pos
}