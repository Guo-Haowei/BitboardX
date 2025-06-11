use crate::{
    board::{bitboard::BitBoard, moves, moves::parse_move, position::Position},
    engine::move_gen::gen_moves,
};
use std::io::{self, Write};

pub mod move_gen;

pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;

pub fn version() -> String {
    format!("{}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
}

pub struct Engine {
    pos: Position,
}

impl Engine {
    pub fn new() -> Self {
        Self { pos: Position::new() }
    }

    pub fn shutdown(&mut self) {
        // Placeholder for any cleanup logic if needed
    }

    pub fn cmd_uci(&self, out: &mut io::Stdout) {
        writeln!(out, "id name MyEngine").unwrap();
        writeln!(out, "id author You").unwrap();
        writeln!(out, "uciok").unwrap();
    }

    pub fn cmd_isready(&self, out: &mut io::Stdout) {
        writeln!(out, "readyok").unwrap();
    }

    pub fn cmd_position(&mut self, _out: &mut io::Stdout, args: &str) {
        let mut parts: Vec<&str> = args.split_whitespace().collect();

        if parts.is_empty() {
            eprintln!("Error: No position command provided"); // @TODO: usage
            return;
        }

        match parts.as_slice() {
            ["startpos", _rest @ ..] => {
                self.pos = Position::new();
                parts.remove(0);
            }
            ["fen", p1, p2, p3, p4, p5, p6, _rest @ ..] => match Position::from(p1, p2, p3, p4, p5, p6) {
                Ok(pos) => {
                    self.pos = pos;
                    parts.drain(0..=6); // remove the FEN parts
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    return;
                }
            },
            _ => {
                eprintln!("Error: Invalid position command");
                return;
            }
        }

        println!("{}", self.pos.state.to_string(true));

        if !parts.is_empty() {
            match parts.as_slice() {
                ["moves", moves @ ..] => {
                    for move_str in moves {
                        match parse_move(move_str) {
                            None => {
                                eprintln!("Error: Invalid move format '{}'", move_str);
                            }
                            // @TODO: refactor this
                            Some((from, to)) => {
                                eprintln!("from: {}, to: {}", from, to);
                                if (gen_moves(&self.pos, from) & BitBoard::from_bit(to)).is_empty() {
                                    eprintln!("Error: Invalid move '{}'", move_str);
                                    break;
                                }
                                let m = match moves::create_move(&self.pos, from, to) {
                                    Some(m) => m,
                                    None => break,
                                };

                                moves::do_move(&mut self.pos, &m);
                                println!("{}", self.pos.state.to_string(true));
                            }
                        }
                    }
                }
                _ => {
                    eprintln!("Warning: Unrecognized position command parts: {:?}", parts);
                }
            }
        }
        // check if there are any moves
    }

    pub fn cmd_go(&self, out: &mut io::Stdout, args: &str) {
        // Placeholder for search logic
        writeln!(out, "TODO: go {}", args).unwrap();
    }
}
