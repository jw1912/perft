#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

mod consts;
mod attacks;
pub mod movegen;
pub mod position;

use consts::Right;
use position::Position;
use std::time::Instant;

const POSITIONS: [(&str, u8, u64); 5] = [
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119_060_324),
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193_690_690,),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 7, 178_633_661),
    ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194),
    ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 5, 164_075_551),
];

fn main() {
    let initial = Instant::now();
    let mut total: u64 = 0;
    for (fen, d, exp) in POSITIONS {
        let pos = parse_fen(fen);
        println!("Position: {fen}");

        let now = Instant::now();
        let count = perft(&pos, d);
        total += count;
        assert_eq!(count, exp);

        let dur = now.elapsed();
        println!(
            "depth {d} time {} nodes {count} Mnps {:.2}\n",
            dur.as_millis(),
            count as f64 / dur.as_micros() as f64
        );
    }

    let dur = initial.elapsed();
    println!(
        "total time {} nodes {total} nps {:.3}",
        dur.as_millis(),
        total as f64 / dur.as_micros() as f64
    );
}

#[must_use]
pub fn perft(pos: &Position, depth: u8) -> u64 {
    let mut tmp;
    let mut positions = 0;
    let moves = pos.gen();
    for m_idx in 0..moves.len {
        tmp = *pos;
        if tmp.make(moves.list[m_idx]) {
            continue;
        }

        positions += if depth > 1 {
            perft(&tmp, depth - 1)
        } else {
            1
        };
    }
    positions
}

#[must_use]
pub fn parse_fen(fen: &str) -> Position {
    let mut pos = Position::default();
    let vec: Vec<&str> = fen.split_whitespace().collect();
    let p: Vec<char> = vec[0].chars().collect();

    // board
    let (mut row, mut col) = (7, 0);
    for ch in p {
        if ch == '/' {
            row -= 1;
            col = 0;
        } else if ('1'..='8').contains(&ch) {
            col += ch.to_string().parse::<i16>().unwrap_or(0);
        } else {
            let idx: usize = "PNBRQKpnbrqk"
                .chars()
                .position(|element| element == ch)
                .unwrap_or(6);
            let colour = usize::from(idx > 5);
            pos.toggle(colour, idx + 2 - 6 * colour, 1 << (8 * row + col));
            col += 1;
        }
    }

    // side to move
    pos.side = vec[1] == "b";

    // castle rights
    for ch in vec[2].chars() {
        pos.rights |= match ch {
            'Q' => Right::WQS,
            'K' => Right::WKS,
            'q' => Right::BQS,
            'k' => Right::BKS,
            _ => 0,
        }
    }

    // en passant square
    pos.enp_sq = if vec[3] == "-" {
        0
    } else {
        let chs: Vec<char> = vec[3].chars().collect();
        8 * chs[1].to_string().parse::<u8>().unwrap_or(0) + chs[0] as u8 - 105
    };

    pos
}
