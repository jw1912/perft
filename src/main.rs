#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

mod consts;
mod attacks;
pub mod movegen;
pub mod position;

use std::{fs::File, io::{BufRead, BufReader}, time::Instant};
use consts::Right;
use position::Position;

struct PerftResult {
    pos: Position,
    fen: String,
    results: Vec<u64>,
}

impl PerftResult {
    fn from_epd(epd: &str) -> Self {
        let split = epd.split(';').collect::<Vec<&str>>();
        let mut result = Self {
            pos: parse_fen(split[0]),
            fen: String::from(split[0]),
            results: Vec::new(),
        };

        for depth in &split[1..] {
            result.results.push(depth.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0));
        }

        result
    }
}

fn main() {
    let mut positions = Vec::new();
    let file = File::open("perft_results.txt").unwrap();
    for line in BufReader::new(file).lines().map(Result::unwrap) {
        positions.push(PerftResult::from_epd(&line));
    }

    let initial = Instant::now();
    let mut total: u64 = 0;

    for PerftResult {pos, fen, results} in positions {
        println!("{fen}");
        for (d, &res) in results.iter().enumerate() {
            let count = perft::<false, true>(&pos, d as u8 + 1);
            total += count;
            assert_eq!(count, res);
        }
    }

    let dur = initial.elapsed();
    println!(
        "total time {} nodes {total} nps {:.3}",
        dur.as_millis(),
        total as f64 / dur.as_micros() as f64
    );
}

#[must_use]
pub fn perft<const ROOT: bool, const BULK: bool>(pos: &Position, depth: u8) -> u64 {
    let moves = pos.gen();

    if BULK && !ROOT && depth == 1 {
        return moves.len as u64;
    }

    let mut tmp;
    let mut positions = 0;
    let leaf = depth == 1;

    for m_idx in 0..moves.len {
        tmp = *pos;
        tmp.make(moves.list[m_idx]);

        let num = if !BULK && leaf {1} else {perft::<false, BULK>(&tmp, depth - 1)};
        positions += num;

        if ROOT {
            println!("{}: {num}", moves.list[m_idx].to_uci());
        }
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
