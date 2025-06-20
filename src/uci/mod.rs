use crate::core::position::Position;
use crate::engine::Engine;
use std::io::{self, Write};

pub struct UCI {
    engine: Engine,
}

impl UCI {
    pub fn new() -> Self {
        Self { engine: Engine::new() }
    }

    pub fn shutdown(&mut self) {}

    pub fn command_uci(&self, out: &mut io::Stdout) {
        writeln!(out, "id name {}", Engine::name()).unwrap();
        writeln!(out, "id author haguo").unwrap();
        writeln!(out, "uciok").unwrap();
    }

    pub fn command_ucinewgame(&mut self, _out: &mut io::Stdout) {
        self.engine = Engine::new();

        // self.pos = Position::new();
        // writeln!(out, "ucinewgame").unwrap();
    }

    pub fn command_isready(&self, out: &mut io::Stdout) {
        writeln!(out, "readyok").unwrap();
    }

    pub fn command_position(&mut self, _out: &mut io::Stdout, args: &str) {
        let mut parts: Vec<&str> = args.split_whitespace().collect();

        if parts.is_empty() {
            eprintln!("Error: No position command provided"); // @TODO: usage
            return;
        }

        match parts.as_slice() {
            ["startpos", _rest @ ..] => {
                self.engine.set_position(Position::new());
                parts.remove(0);
            }
            ["fen", p1, p2, p3, p4, p5, p6, _rest @ ..] => {
                let result = [*p1, *p2, *p3, *p4, *p5, *p6].join(" ");
                match Position::from_fen(result.as_str()) {
                    Ok(pos) => {
                        self.engine.set_position(pos);
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
                        if !self.engine.make_move(move_str) {
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

    pub fn command_go(&mut self, out: &mut io::Stdout, args: &str) {
        let parts: Vec<&str> = args.split_whitespace().collect();

        match parts.as_slice() {
            ["perft", p1, _rest @ ..] => {
                let depth: u8 = match p1.parse() {
                    Ok(d) if d <= 8 => d,
                    _ => {
                        eprintln!("Error: Invalid depth '{}'. Must be between 0 and 8.", p1);
                        return;
                    }
                };
                self.engine.uci_go_perft(out, depth, depth);
            }
            _ => {
                let mv = self.engine.best_move(4).unwrap();
                writeln!(out, "bestmove {}", mv.to_string()).unwrap();
            }
        }
    }

    pub fn command_d(&self, out: &mut io::Stdout) {
        self.engine.uci_cmd_d(out);
    }
}
