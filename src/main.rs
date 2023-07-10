#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

mod consts;
mod attacks;
pub mod movegen;
pub mod position;

use std::{fs::File, io::{BufRead, BufReader}, time::Instant};
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
            pos: Position::parse_fen(split[0]),
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
