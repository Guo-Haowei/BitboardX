pub mod undo_redo;

use crate::engine::board::Square;
use crate::engine::moves;
use crate::engine::position::Position;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    pos: Rc<RefCell<Position>>,
    manager: undo_redo::CommandManager,
}

struct MoveCommand {
    pos: Rc<RefCell<Position>>,
    move_: moves::Move,
}

impl undo_redo::Command for MoveCommand {
    fn execute(&mut self) {
        moves::do_move(&mut self.pos.borrow_mut(), &self.move_);
    }

    fn undo(&mut self) {
        moves::undo_move(&mut self.pos.borrow_mut(), &self.move_);
    }
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

        Self { pos: Rc::new(RefCell::new(pos)), manager: undo_redo::CommandManager::new() }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.pos.borrow().to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.pos.borrow().to_board_string()
    }

    pub fn can_undo(&self) -> bool {
        self.manager.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.manager.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        self.manager.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.manager.redo()
    }

    pub fn execute(&mut self, move_str: &str) -> bool {
        let squares = moves::parse_move(move_str);
        if squares.is_none() {
            return false;
        }

        let (from, to) = squares.unwrap();
        let move_ = self.pos.borrow_mut().legal_move_from_to(from, to);
        if move_.is_none() {
            return false;
        }

        let cmd = Box::new(MoveCommand { pos: Rc::clone(&self.pos), move_: move_.unwrap() });
        self.manager.execute(cmd);

        true
    }

    pub fn legal_move(&self, sq: u8) -> u64 {
        // self.pos.borrow_mut().pseudo_legal_move(square).get()
        self.pos.borrow_mut().legal_move(Square(sq)).get()
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
        let mut game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(game.execute("e2e4")); // e2 to e4
        assert_eq!(
            game.to_board_string(),
            "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR"
        );
        game.undo();
        assert_eq!(
            game.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
        game.redo();
        assert_eq!(
            game.to_board_string(),
            "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR"
        );
    }
}
