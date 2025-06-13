use super::position::Position;
use std::io::{self, Write};
use wasm_bindgen::prelude::*;

pub const NAME: &str = "BitboardX";
/// version: 0.1.5: en passant legal move generation
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 5;

pub fn version() -> String {
    format!("{}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
}

#[wasm_bindgen]
pub fn name() -> String {
    format!("{} {}", NAME, version())
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
        writeln!(out, "id name {}", name()).unwrap();
        writeln!(out, "id author haguo").unwrap();
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
            ["fen", p1, p2, p3, p4, p5, p6, _rest @ ..] => {
                match Position::from_parts(p1, p2, p3, p4, p5, p6) {
                    Ok(pos) => {
                        self.pos = pos;
                        parts.drain(0..=6); // remove the FEN parts
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        return;
                    }
                }
            }
            _ => {
                eprintln!("Error: Invalid position command");
                return;
            }
        }

        if !parts.is_empty() {
            match parts.as_slice() {
                ["moves", moves @ ..] => {
                    for move_str in moves {
                        if !self.pos.apply_move_str(move_str) {
                            eprintln!("Error: Invalid move '{}'", move_str);
                            break;
                        }
                    }
                }
                _ => {
                    eprintln!("Warning: Unrecognized position command parts: {:?}", parts);
                }
            }
        }
    }

    pub fn cmd_go(&self, out: &mut io::Stdout, args: &str) {
        // Placeholder for search logic
        writeln!(out, "TODO: go {}", args).unwrap();
    }
}
