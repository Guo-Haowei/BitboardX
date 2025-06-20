use std::collections::HashMap;
use std::io::{self, Write};

use crate::core::{move_gen, utils, zobrist};
use crate::core::{position::Position, types::Move, zobrist::Zobrist};
use crate::engine::search;
use crate::logger;

const NAME: &str = "BitboardX";
const VERSION_MAJOR: u32 = 0;
const VERSION_MINOR: u32 = 1;
const VERSION_PATCH: u32 = 3; // pesto

pub struct Engine {
    pub(super) pos: Position,
    pub(super) history: HashMap<Zobrist, u32>, // for threefold detection
    pub(super) last_hash: Zobrist,
}

impl Engine {
    pub fn name() -> String {
        format!("{} {}", NAME, Self::version())
    }

    pub fn version() -> String {
        format!("{}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
    }

    pub fn new() -> Self {
        Self::from_fen(Position::DEFAULT_FEN).unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let pos = Position::from_fen(fen)?;
        let last_hash = zobrist::zobrist_hash(&pos);
        let mut history = HashMap::new();
        history.insert(last_hash, 1);

        Ok(Self { pos, history, last_hash })
    }

    pub fn best_move(&mut self, depth: u8) -> Option<Move> {
        search::find_best_move(self, depth)
    }

    pub fn set_position(&mut self, pos: Position) {
        let zobrist = zobrist::zobrist_hash(&pos);
        self.pos = pos;

        self.history.clear();
        self.history.insert(zobrist, 1);
        self.last_hash = zobrist;
    }

    // Assume that the move is legal, otherwise it might crash the engine
    pub fn make_move(&mut self, mv: &str) -> bool {
        let mv = utils::parse_move(mv);
        if mv.is_none() {
            return false;
        }

        let mv = mv.unwrap();
        let legal_moves = move_gen::legal_moves(&self.pos);
        let src_sq = mv.src_sq();
        let dst_sq = mv.dst_sq();
        let promotion = mv.get_promotion();
        for mv in legal_moves.iter() {
            if mv.src_sq() == src_sq && mv.dst_sq() == dst_sq && mv.get_promotion() == promotion {
                self.make_move_unverified(mv.clone());
                return true;
            }
        }

        return false;
    }

    pub fn make_move_unverified(&mut self, mv: Move) {
        self.pos.make_move(mv);

        let zobrist = zobrist::zobrist_hash(&self.pos);
        self.last_hash = zobrist;
        *self.history.entry(zobrist).or_insert(0) += 1;
    }

    /// The following methods are for UCI commands
    pub fn handle_uci_cmd(&mut self, out: &mut io::Stdout, input: &str) -> bool {
        let mut parts = input.splitn(2, ' ');
        let cmd = parts.next().unwrap();
        let args = parts.next().unwrap_or("");

        match cmd {
            "uci" => self.uci_cmd_uci(out),
            "ucinewgame" => self.uci_cmd_ucinewgame(out),
            "isready" => self.uci_cmd_isready(out),
            "position" => self.uci_cmd_position(out, args),
            "go" => self.uci_cmd_go(out, args),
            "d" => self.uci_cmd_d(out),
            "q" | "quit" => {
                // @TODO: shutdown
                return false;
            }
            _ => {
                logger::log(
                    format!("Unknown command: '{}'. Type help for more information.", input)
                        .to_string(),
                );
            }
        }

        true
    }

    pub fn uci_cmd_isready(&self, out: &mut io::Stdout) {
        writeln!(out, "readyok").unwrap();
    }

    pub fn uci_cmd_ucinewgame(&mut self, _out: &mut io::Stdout) {
        panic!("UCI command 'ucinewgame' is not implemented yet");
    }

    pub fn uci_cmd_uci(&self, out: &mut io::Stdout) {
        writeln!(out, "id name {}", Engine::name()).unwrap();
        writeln!(out, "id author haguo").unwrap();
        writeln!(out, "uciok").unwrap();
    }

    pub fn uci_cmd_d(&self, out: &mut io::Stdout) {
        writeln!(out, "{}", utils::debug_string(&self.pos)).unwrap();
    }

    pub fn uci_cmd_position(&mut self, _out: &mut io::Stdout, args: &str) {
        let mut parts: Vec<&str> = args.split_whitespace().collect();

        if parts.is_empty() {
            logger::log("Error: position command requires arguments".to_string());
            return;
        }

        match parts.as_slice() {
            ["startpos", _rest @ ..] => {
                self.set_position(Position::new());
                parts.remove(0);
            }
            ["fen", p1, p2, p3, p4, p5, p6, _rest @ ..] => {
                let result = [*p1, *p2, *p3, *p4, *p5, *p6].join(" ");
                match Position::from_fen(result.as_str()) {
                    Ok(pos) => {
                        self.set_position(pos);
                        parts.drain(0..=6); // remove the FEN parts
                    }
                    Err(err) => {
                        logger::log(format!("Error: {}", err));
                        return;
                    }
                }
            }
            _ => {
                logger::log("Error: Invalid position command".to_string());
                return;
            }
        }

        if !parts.is_empty() {
            match parts.as_slice() {
                ["moves", moves @ ..] => {
                    for move_str in moves {
                        if !self.make_move(move_str) {
                            logger::log(format!("Error: Invalid move '{}'", move_str));
                            break;
                        }
                    }
                }
                _ => {
                    logger::log(format!(
                        "Warning: Unrecognized position command parts: {:?}",
                        parts
                    ));
                }
            }
        }
    }

    pub fn uci_cmd_go(&mut self, out: &mut io::Stdout, args: &str) {
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
                self.uci_cmd_go_perft(out, depth, depth);
            }
            _ => {
                let mv = self.best_move(4).unwrap();
                writeln!(out, "bestmove {}", mv.to_string()).unwrap();
            }
        }
    }

    fn uci_cmd_go_perft(&mut self, out: &mut io::Stdout, depth: u8, max_depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let move_list = move_gen::legal_moves(&self.pos);

        let mut nodes = 0u64;
        let should_print = depth == max_depth;
        for mv in move_list.iter() {
            let undo_state = self.pos.make_move(mv.clone());
            let count = self.uci_cmd_go_perft(out, depth - 1, max_depth);
            nodes += count;
            self.pos.unmake_move(mv.clone(), &undo_state);

            if should_print {
                writeln!(out, "{}: {}", mv.to_string(), count).unwrap();
            }
        }

        if should_print {
            writeln!(out, "\nNodes searched: {}", nodes).unwrap();
        }

        nodes
    }
}
