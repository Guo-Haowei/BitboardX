use bitboard_x::engine::{move_gen, name, position::Position, utils};
use rustyline::{DefaultEditor, Result};
use std::io::{self, Write};

pub struct UCI {
    pos: Position,
}

impl UCI {
    pub fn new() -> Self {
        Self { pos: Position::new() }
    }

    pub fn shutdown(&mut self) {}

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
                let result = [*p1, *p2, *p3, *p4, *p5, *p6].join(" ");
                match Position::from_fen(result.as_str()) {
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

    pub fn cmd_go(&mut self, _out: &mut io::Stdout, args: &str) {
        let parts: Vec<&str> = args.split_whitespace().collect();

        if parts.is_empty() {
            eprintln!("Error: No go command provided"); // @TODO: usage
            return;
        }

        match parts.as_slice() {
            ["perft", p1, _rest @ ..] => {
                let depth: u8 = match p1.parse() {
                    Ok(d) if d <= 8 => d,
                    _ => {
                        eprintln!("Error: Invalid depth '{}'. Must be between 0 and 8.", p1);
                        return;
                    }
                };
                perft_test(&mut self.pos, depth, depth);
            }
            _ => {
                eprintln!("Error: Invalid go command");
                return;
            }
        }
    }

    pub fn cmd_d(&self, out: &mut io::Stdout) {
        writeln!(out, "{}", utils::debug_string(&self.pos)).unwrap();
    }
}

fn perft_test(pos: &mut Position, depth: u8, max_depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let move_list = move_gen::legal_moves(pos);

    let mut nodes = 0u64;
    let should_print = depth == max_depth;
    for m in move_list.iter() {
        let undo_state = pos.make_move(m.clone());
        let count = perft_test(pos, depth - 1, max_depth);
        nodes += count;
        pos.unmake_move(m.clone(), &undo_state);

        if should_print {
            eprintln!("{}: {}", m.to_string(), count);
        }
    }

    if should_print {
        eprintln!("\nNodes searched: {}", nodes);
    }

    nodes
}

fn main() -> Result<()> {
    eprintln!("{}", name());
    let mut stdout = io::stdout();
    let mut uci = UCI::new();
    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history("history.txt").is_err();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let input = line.trim();

                if !input.is_empty() {
                    let _ = rl.add_history_entry(line.as_str());
                }

                let mut parts = input.splitn(2, ' ');
                let cmd = parts.next().unwrap();
                let args = parts.next().unwrap_or("");

                match cmd {
                    "uci" => uci.cmd_uci(&mut stdout),
                    "isready" => uci.cmd_isready(&mut stdout),
                    "position" => uci.cmd_position(&mut stdout, args),
                    "go" => uci.cmd_go(&mut stdout, args),
                    "d" => uci.cmd_d(&mut stdout),
                    "q" | "quit" => {
                        uci.shutdown();
                        break;
                    }
                    _ => eprintln!("Unknown command: '{}'. Type help for more information.", input),
                }
            }
            _ => {
                break;
            }
        }
    }

    rl.save_history("history.txt")?;
    Ok(())
}
