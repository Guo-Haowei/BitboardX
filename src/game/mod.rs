use crate::engine::board::Square;
use crate::engine::position::Position;
use crate::engine::utils;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    pos: Position,
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

        Self { pos }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.pos.to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.pos.to_board_string()
    }

    pub fn can_undo(&self) -> bool {
        self.pos.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.pos.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        self.pos.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.pos.redo()
    }

    pub fn execute(&mut self, move_str: &str) -> bool {
        let squares = utils::parse_move(move_str);
        if squares.is_none() {
            return false;
        }

        let (from, to) = squares.unwrap();
        let move_ = self.pos.legal_move_from_to(from, to);
        if move_.is_none() {
            return false;
        }

        self.pos.do_move(&move_.unwrap());

        true
    }

    pub fn legal_move(&mut self, sq: u8) -> u64 {
        // self.pos.borrow_mut().pseudo_legal_move(square).get()
        self.pos.legal_move(Square(sq)).get()
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
