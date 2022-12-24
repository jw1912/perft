mod consts;
mod position;
mod movegen;

use consts::*;
use position::Position;
use movegen::MoveList;
use std::time::{Instant, Duration};

const POSITIONS: [(&str, u8, u64); 5] = [
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119_060_324),
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 7, 178_633_661),
    ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194),
    ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551),
];

fn main() {
    let initial: Instant = Instant::now();
    let mut total: u64 = 0;
    for (fen, d, exp) in POSITIONS {
        let pos: Position = parse_fen(fen);
        println!("Position: {fen}");
        let now: Instant = Instant::now();
        let count: u64 = perft(&pos, d);
        total += count;
        assert_eq!(count, exp);
        let dur: Duration = now.elapsed();
        println!("depth {} time {} nodes {count} Mnps {:.2}\n", d, dur.as_millis(), count as f64 / dur.as_micros() as f64);
    }
    let dur: Duration = initial.elapsed();
    println!("total time {} nodes {} nps {:.3}", dur.as_millis(), total, total as f64 / dur.as_micros() as f64)
}

fn perft(pos: &Position, depth_left: u8) -> u64 {
    let mut moves: MoveList = MoveList::default();
    let mut tmp: Position;
    let mut positions: u64 = 0;
    pos.gen(&mut moves);
    for m_idx in 0..moves.len {
        tmp = *pos;
        if tmp.do_move(moves.list[m_idx]) { continue }
        positions += if depth_left > 1 {perft(&tmp, depth_left - 1)} else {1};
    }
    positions
}

fn parse_fen(fen: &str) -> Position {
    let mut pos: Position = Position { board: [XX; 120], c: false, enp: 0, cr: 0 };
    let vec: Vec<&str> = fen.split_whitespace().collect();
    let p: Vec<char> = vec[0].chars().collect();
    let (mut row, mut col): (i16, u16) = (7, 0);
    for ch in p {
        if ch == '/' { row -= 1; col = 0; }
        else if ('1'..='8').contains(&ch) {
            let empties: u16 = ch.to_string().parse::<u16>().unwrap_or(0);
            let idx_64: u16 = 8 * row as u16 + col;
            col += empties;
            for i in 0..empties {pos.set_square(idx_64 + i, E)}
        } else {
            let val: usize = [' ','P','N','B','R','Q','K',' ',' ','p','n','b','r','q','k'].iter().position(|&element| element == ch).unwrap_or(6);
            let idx_64: u16 = 8 * row as u16 + col;
            pos.set_square(idx_64, val as u8);
            col += 1;
        }
    }
    pos.c = vec[1] == "b";
    for ch in vec[2].chars() {pos.cr |= match ch {'Q' => WQS, 'K' => WKS, 'q' => BQS, 'k' => BKS, _ => 0}}
    pos.enp = if vec[3] == "-" {0} else {
        let chs: Vec<char> = vec[3].chars().collect();
        8 * chs[1].to_string().parse::<u16>().unwrap_or(0) + chs[0] as u16 - 105
    };
    pos
}