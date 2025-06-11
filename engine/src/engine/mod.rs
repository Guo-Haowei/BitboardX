use crate::board::position::Position;
use std::io::{self, Write};

pub mod move_gen;

pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;

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

    pub fn cmd_position(&mut self, out: &mut io::Stdout, fen: &String) {
        match Position::from_fen(fen) {
            Ok(pos) => self.pos = pos,
            Err(err) => eprintln!("Error parsing FEN: {}", err),
        }
    }

    pub fn cmd_go(&self, out: &mut io::Stdout) {
        // Placeholder for search logic
        writeln!(out, "searching for best move...").unwrap();
    }
}
