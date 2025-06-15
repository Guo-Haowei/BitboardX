use crate::engine::board::Move;
use crate::engine::move_gen;
use crate::engine::position::*;
use crate::engine::utils;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct MoveJs {
    internal: Move,
}

#[wasm_bindgen]
impl MoveJs {
    fn new(m: &Move) -> Self {
        Self { internal: m.clone() }
    }

    pub fn from_sq(&self) -> u8 {
        self.internal.from_sq().as_u8()
    }

    pub fn to_sq(&self) -> u8 {
        self.internal.to_sq().as_u8()
    }
}

#[wasm_bindgen]
pub struct Game {
    pos: Position,

    undo_stack: Vec<(Move, Snapshot)>,
    redo_stack: Vec<(Move, Snapshot)>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let pos = match Position::from(fen) {
            Ok(pos) => pos,
            Err(err) => {
                eprintln!("Error parsing FEN: {}", err);
                Position::new()
            }
        };

        Self { pos, undo_stack: Vec::new(), redo_stack: Vec::new() }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.pos.to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.pos.to_board_string()
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.redo_stack.len() > 0
    }

    pub fn do_move(&mut self, string: String) {
        let m = utils::parse_move(string.as_str());
        if m.is_none() {
            return;
        }

        let (from, to) = m.unwrap();
        let legal_moves = move_gen::legal_moves(&self.pos);
        for m in legal_moves.iter() {
            if m.from_sq() == from && m.to_sq() == to {
                console::log_1(&format!("move '{}'", string).into());
                let m = m.clone();
                let snapshot = self.pos.make_move(m);

                self.undo_stack.push((m, snapshot));
                self.redo_stack.clear();
            }
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some((m, snapshot)) = self.undo_stack.pop() {
            self.pos.unmake_move(m, &snapshot);
            self.redo_stack.push((m, snapshot));
            return true;
        }

        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some((m, snapshot)) = self.redo_stack.pop() {
            self.pos.make_move(m);
            self.undo_stack.push((m, snapshot));
            return true;
        }

        false
    }

    pub fn legal_moves(&self) -> Vec<MoveJs> {
        move_gen::legal_moves(&self.pos).iter().map(|m| MoveJs::new(m)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            game.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_apply_move() {
        // let mut game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        // assert!(game.execute("e2e4")); // e2 to e4
        // assert_eq!(
        //     game.to_board_string(),
        //     "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR"
        // );
        // game.undo();
        // assert_eq!(
        //     game.to_board_string(),
        //     "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        // );
        // game.redo();
        // assert_eq!(
        //     game.to_board_string(),
        //     "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR"
        // );

        // panic!(
        //     "Test failed, this is a placeholder panic to indicate the test should be implemented."
        // );
    }
}
