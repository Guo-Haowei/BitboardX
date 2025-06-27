use std::collections::HashMap;

use crate::core::{
    position::{Position, UndoState},
    types::*,
    zobrist::ZobristHash,
};

pub struct GameState {
    pub pos: Position,

    repetition_table: HashMap<ZobristHash, u32>, // for threefold detection
}

impl GameState {
    pub fn new() -> Self {
        Self::from_fen(Position::DEFAULT_FEN).unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let pos = Position::from_fen(fen)?;
        let mut repetition_table = HashMap::new();
        assert!(pos.state.hash != ZobristHash(0));
        repetition_table.insert(pos.state.hash, 1);

        Ok(Self { pos, repetition_table })
    }

    pub fn set_position(&mut self, pos: Position) {
        self.pos = pos;
        self.repetition_table.clear();
        assert!(pos.state.hash != ZobristHash(0));
        self.repetition_table.insert(pos.state.hash, 1);
    }

    pub fn make_move(&mut self, mv: Move) -> UndoState {
        let undo_state = self.pos.make_move(mv);
        let hash = self.pos.state.hash;
        *self.repetition_table.entry(hash).or_insert(0) += 1;

        undo_state
    }

    pub fn unmake_move(&mut self, mv: Move, undo_state: &UndoState) {
        let hash = self.pos.state.hash;
        let entry = self.repetition_table.get_mut(&hash).unwrap();
        *entry -= 1;

        self.pos.unmake_move(mv, undo_state);
    }

    pub fn is_three_fold(&self) -> bool {
        let count = *self.repetition_table.get(&self.pos.state.hash).unwrap_or(&0);
        assert!(count <= 3);
        count >= 3
    }

    pub fn is_fifty_draw(&self) -> bool {
        assert!(self.pos.state.halfmove_clock <= 100);
        self.pos.state.halfmove_clock >= 100
    }
}
