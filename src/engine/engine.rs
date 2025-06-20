use std::collections::HashMap;

use crate::core::{move_gen, utils, zobrist};
use crate::core::{position::Position, types::Move, zobrist::Zobrist};
use crate::engine::search;

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

    pub fn best_move(&mut self, depth: u8) -> Option<Move> {
        search::find_best_move(self, depth)
    }
}
