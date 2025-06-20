use std::collections::HashMap;

use crate::core::{position::Position, types::Move, zobrist::Zobrist};
use crate::engine::search::*;

const NAME: &str = "BitboardX";
const VERSION_MAJOR: u32 = 0;
const VERSION_MINOR: u32 = 1;
const VERSION_PATCH: u32 = 3; // pesto

pub struct Engine {
    pub pos: Position,
    pub history: HashMap<Zobrist, u32>,
}

impl Engine {
    pub fn new() -> Self {
        let pos = Position::new();
        let history = HashMap::new();

        Engine { pos, history }
    }

    pub fn name() -> String {
        format!("{} {}", NAME, Self::version())
    }

    pub fn version() -> String {
        format!("{}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
    }

    pub fn set_position(&mut self, pos: Position) {
        self.pos = pos;
    }

    pub fn best_move(&mut self, depth: u8) -> Option<Move> {
        find_best_move(self, depth)
    }
}
