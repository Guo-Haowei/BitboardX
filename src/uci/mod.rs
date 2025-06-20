use crate::core::{move_gen, position::Position, utils};
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
                panic!("Perft test is not implemented yet");
                // perft_test(&mut self.engine.pos, depth, depth);
            }
            _ => {}
        }

        let mv = self.engine.best_move(4).unwrap();
        writeln!(out, "bestmove {}", mv.to_string()).unwrap();
    }

    pub fn command_d(&self, out: &mut io::Stdout) {
        panic!("Perft test is not implemented yet");
        // writeln!(out, "{}", utils::debug_string(&self.engine.pos)).unwrap();
    }
}

fn perft_test(pos: &mut Position, depth: u8, max_depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let move_list = move_gen::legal_moves(pos);

    let mut nodes = 0u64;
    let should_print = depth == max_depth;
    for mv in move_list.iter() {
        let undo_state = pos.make_move(mv.clone());
        let count = perft_test(pos, depth - 1, max_depth);
        nodes += count;
        pos.unmake_move(mv.clone(), &undo_state);

        if should_print {
            eprintln!("{}: {}", mv.to_string(), count);
        }
    }

    if should_print {
        eprintln!("\nNodes searched: {}", nodes);
    }

    nodes
}
