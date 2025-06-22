use std::collections::HashMap;
use std::io::Write;

use crate::core::{move_gen, position::Position, types::Move, zobrist::ZobristHash};
use crate::engine::search;
use crate::engine::ttable::TTable;
use crate::utils;

const NAME: &str = "BitboardX";
const VERSION_MAJOR: u32 = 0;
const VERSION_MINOR: u32 = 1;
const VERSION_PATCH: u32 = 7;

pub struct Engine {
    pub(super) pos: Position,
    pub(super) repetition_table: HashMap<ZobristHash, u32>, // for threefold detection
    pub(super) last_hash: ZobristHash,
    pub(super) tt: TTable,
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
        let last_hash = pos.zobrist();
        let mut repetition_table = HashMap::new();
        repetition_table.insert(last_hash, 1);

        Ok(Self { pos, repetition_table, last_hash, tt: TTable::new() })
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn best_move(&mut self, depth: u8) -> Option<Move> {
        let mut searcher = search::Searcher::new();
        searcher.find_best_move(self, depth)
    }

    pub fn set_position(&mut self, pos: Position) {
        let zobrist = pos.zobrist();
        self.pos = pos;
        self.repetition_table.clear();
        self.repetition_table.insert(zobrist, 1);
        self.last_hash = zobrist;
    }

    // Assume that the move is legal, otherwise it might crash the engine
    pub fn make_move(&mut self, mv_str: &str) -> bool {
        let mv = utils::parse_move(mv_str);
        if mv.is_none() {
            log::error!("Failed to parse move: '{}'", mv_str);
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

        log::error!("'{}' is not a legal move", mv_str);
        return false;
    }

    pub fn repetition_count(&self, key: ZobristHash) -> u32 {
        let val = self.repetition_table.get(&key).unwrap_or(&0);
        *val
    }

    pub fn make_move_unverified(&mut self, mv: Move) {
        self.pos.make_move(mv);

        let zobrist = self.pos.zobrist();
        self.last_hash = zobrist;
        *self.repetition_table.entry(zobrist).or_insert(0) += 1;
    }

    /// The following methods are for UCI commands
    pub fn handle_uci_cmd<W: Write>(&mut self, writer: &mut W, input: &str) -> bool {
        let mut parts = input.splitn(2, ' ');
        let cmd = parts.next().unwrap();
        let args = parts.next().unwrap_or("");

        match cmd {
            "uci" => self.uci_cmd_uci(writer),
            "ucinewgame" => self.uci_cmd_ucinewgame(writer),
            "isready" => self.uci_cmd_isready(writer),
            "position" => self.uci_cmd_position(writer, args),
            "go" => self.uci_cmd_go(writer, args),
            "d" => self.uci_cmd_d(writer),
            "q" | "quit" => {
                // @TODO: shutdown
                return false;
            }
            _ => {
                log::error!("Unknown command: '{}'. Type help for more information.", input);
            }
        }

        true
    }

    pub fn uci_cmd_isready<W: Write>(&self, writer: &mut W) {
        writeln!(writer, "readyok").unwrap();
    }

    pub fn uci_cmd_ucinewgame<W: Write>(&mut self, _: &mut W) {
        self.set_position(Position::new());
    }

    pub fn uci_cmd_uci<W: Write>(&self, writer: &mut W) {
        writeln!(writer, "id name {}", Engine::name()).unwrap();
        writeln!(writer, "id author haguo").unwrap();
        writeln!(writer, "uciok").unwrap();
    }

    pub fn uci_cmd_d<W: Write>(&self, writer: &mut W) {
        writeln!(writer, "{}", utils::debug_string(&self.pos)).unwrap();
    }

    pub fn uci_cmd_position<W: Write>(&mut self, _: &mut W, args: &str) {
        let mut parts: Vec<&str> = args.split_whitespace().collect();

        if parts.is_empty() {
            log::error!("Error: position command requires arguments");
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
                        log::error!("Error: {}", err);
                        return;
                    }
                }
            }
            _ => {
                log::error!("Error: Invalid position command");
                return;
            }
        }

        if !parts.is_empty() {
            match parts.as_slice() {
                ["moves", moves @ ..] => {
                    for move_str in moves {
                        if !self.make_move(move_str) {
                            log::error!("Error: Invalid move '{}'", move_str);
                            break;
                        }
                    }
                }
                _ => {
                    log::error!("Warning: Unrecognized position command parts: {:?}", parts);
                }
            }
        }
    }

    pub fn uci_cmd_go<W: Write>(&mut self, writer: &mut W, args: &str) {
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
                self.uci_cmd_go_perft(writer, depth, depth);
            }
            _ => {
                let mv = self.best_move(4).unwrap();
                writeln!(writer, "bestmove {}", mv.to_string()).unwrap();
            }
        }
    }

    fn uci_cmd_go_perft<W: Write>(&mut self, writer: &mut W, depth: u8, max_depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let move_list = move_gen::legal_moves(&self.pos);

        let mut nodes = 0u64;
        let should_print = depth == max_depth;
        for mv in move_list.iter() {
            let undo_state = self.pos.make_move(mv.clone());
            let count = self.uci_cmd_go_perft(writer, depth - 1, max_depth);
            nodes += count;
            self.pos.unmake_move(mv.clone(), &undo_state);

            if should_print {
                writeln!(writer, "{}: {}", mv.to_string(), count).unwrap();
            }
        }

        if should_print {
            writeln!(writer, "\nNodes searched: {}", nodes).unwrap();
        }

        nodes
    }
}
