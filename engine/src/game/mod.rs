use crate::board::moves;
use crate::board::position::Position;
use crate::engine::move_gen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    pos: Position,
    history: Vec<moves::Move>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let position = match Position::from_fen(fen) {
            Ok(pos) => pos,
            Err(err) => {
                eprintln!("Error parsing FEN: {}", err);
                Position::new()
            }
        };
        Self { pos: position, history: Vec::new() }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.pos.state.to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.pos.state.to_board_string()
    }

    // @TODO: move to moves.rs
    pub fn apply_move(&mut self, from: u8, to: u8) -> bool {
        match moves::apply_move(&mut self.pos, from, to) {
            None => return false,
            Some(m) => {
                self.history.push(m);
                true
            }
        }
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some(last_move) = self.history.pop() {
            moves::undo_move(&mut self.pos, &last_move);
            return true;
        }

        false
    }

    pub fn apply_move_str(&mut self, move_str: &str) -> bool {
        match moves::apply_move_str(&mut self.pos, move_str) {
            None => return false,
            Some(m) => {
                self.history.push(m);
                true
            }
        }
    }

    pub fn gen_moves(&self, square: u8) -> u64 {
        move_gen::gen_moves(&self.pos, square).get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(game.to_board_string(), "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR");
    }

    #[test]
    fn test_apply_move() {
        let mut game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(game.apply_move(SQ_E2, SQ_E4)); // e2 to e4
        assert_eq!(game.to_board_string(), "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR");
        game.undo_move();
        assert_eq!(game.to_board_string(), "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR");
    }
}
