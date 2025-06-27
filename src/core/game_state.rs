use crate::core::{position::Position, zobrist::ZobristHash};

pub struct GameState {
    pub pos: Position,
    pub zobrist_stack: Vec<ZobristHash>,
}

impl GameState {
    pub fn new() -> Self {
        Self::from_fen(Position::DEFAULT_FEN).unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let pos = Position::from_fen(fen)?;
        let mut zobrist_stack = Vec::with_capacity(128);
        zobrist_stack.push(pos.zobrist());

        Ok(Self { pos, zobrist_stack })
    }

    pub fn set_position(&mut self, pos: Position) {
        self.pos = pos;
        self.zobrist_stack.clear();
        self.zobrist_stack.push(self.pos.zobrist());
    }

    pub fn push_zobrist(&mut self) {
        self.zobrist_stack.push(self.pos.zobrist());
    }

    pub fn pop_zobrist(&mut self) {
        let last = self.zobrist_stack.pop();
        debug_assert!(last.is_some());
    }

    pub fn is_three_fold(&self) -> bool {
        let last = self.zobrist_stack.last().unwrap();
        let mut count = 0;
        for zobrist in self.zobrist_stack.iter() {
            if *zobrist == *last {
                count += 1;
            }
            if count >= 3 {
                debug_assert!(count == 3);
                return true;
            }
        }

        false
    }

    pub fn is_fifty_draw(&self) -> bool {
        debug_assert!(self.pos.state.halfmove_clock <= 100);
        self.pos.state.halfmove_clock >= 100
    }
}
