use std::io::Write;

use crate::core::{game_state::GameState, move_gen, position::Position, types::Move};
use crate::engine::search;
use crate::engine::ttable::TTable;
use crate::utils;

const NAME: &str = "BitboardX";
const VERSION_MAJOR: u32 = 0;
const VERSION_MINOR: u32 = 2;
const VERSION_PATCH: u32 = 3;

// need an extra layer to track 50 move rule, and threefold repetition

pub struct Engine {
    pub state: GameState,
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
        let state = GameState::from_fen(fen)?;

        Ok(Self { state, tt: TTable::new() })
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn best_move(&mut self, time: f64) -> Option<Move> {
        let mut searcher = search::Searcher::new(time);
        searcher.find_best_move(self)
    }

    pub fn best_move_depth(&mut self, max_depth: u8) -> Option<Move> {
        let mut searcher = search::Searcher::new(f64::MAX);
        searcher.find_best_move_depth(self, max_depth)
    }

    pub fn apply_move_safe(&mut self, mv_str: &str) -> bool {
        let mv = utils::parse_move(mv_str);
        if mv.is_none() {
            log::error!("Failed to parse move: '{}'", mv_str);
            return false;
        }

        let mv = mv.unwrap();
        let legal_moves = move_gen::legal_moves(&mut self.state.pos);
        let src_sq = mv.src_sq();
        let dst_sq = mv.dst_sq();
        let promotion = mv.get_promotion();
        for mv in legal_moves.iter().copied() {
            if mv.src_sq() == src_sq && mv.dst_sq() == dst_sq && mv.get_promotion() == promotion {
                self.state.pos.make_move(mv);
                self.state.push_zobrist();
                return true;
            }
        }

        return false;
    }

    pub fn set_position(&mut self, args: &str) -> Result<(), &'static str> {
        let mut parts: Vec<&str> = args.split_whitespace().collect();

        if parts.is_empty() {
            return Err("Error: position command requires arguments");
        }

        match parts.as_slice() {
            ["startpos", _rest @ ..] => {
                self.state.set_position(Position::new());
                parts.remove(0);
            }
            ["fen", p1, p2, p3, p4, p5, p6, _rest @ ..] => {
                let result = [*p1, *p2, *p3, *p4, *p5, *p6].join(" ");
                match Position::from_fen(result.as_str()) {
                    Ok(pos) => {
                        self.state.set_position(pos);
                        parts.drain(0..=6); // remove the FEN parts
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            _ => {
                return Err("Invalid position command");
            }
        }

        if !parts.is_empty() {
            match parts.as_slice() {
                ["moves", moves @ ..] => {
                    for move_str in moves {
                        if !self.apply_move_safe(move_str) {
                            return Err("Error: Invalid move in position command");
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn perft_test<W: Write>(&self, writer: &mut W, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut pos = self.state.pos.clone();
        let move_list = move_gen::pseudo_legal_moves(&mut pos);

        let mut nodes = 0;
        if cfg!(target_arch = "wasm32") {
            panic!("Perft tests are not supported in wasm32 builds");
        } else {
            use std::thread;
            let mut handles = Vec::new();

            for mv in move_list.iter().copied() {
                let mut child = pos.clone();
                let (_, ok) = child.make_move(mv);
                if !ok {
                    continue;
                }
                let handle =
                    thread::spawn(move || Self::perft_test_inner(&mut child, depth - 1, mv));
                handles.push(handle);
            }

            // Wait for all threads and sum the results
            for handle in handles {
                let (mv, count) = handle.join().expect("Thread panicked");

                writeln!(writer, "{}: {}", mv.to_string(), count).unwrap();
                nodes += count;
            }
        }

        writeln!(writer, "\nNodes searched: {}", nodes).unwrap();

        nodes
    }

    fn perft_test_inner(pos: &mut Position, depth: u8, mv: Move) -> (Move, u64) {
        if depth == 0 {
            return (mv, 1);
        }

        let move_list = move_gen::pseudo_legal_moves(pos);

        let mut nodes = 0u64;
        for mv in move_list.iter().copied() {
            let (undo_state, ok) = pos.make_move(mv);
            if !ok {
                pos.unmake_move(mv, &undo_state);
                continue;
            }

            let (_, count) = Self::perft_test_inner(pos, depth - 1, mv);
            nodes += count;
            pos.unmake_move(mv, &undo_state);
        }

        (mv, nodes)
    }
}
