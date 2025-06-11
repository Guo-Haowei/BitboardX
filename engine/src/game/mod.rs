use crate::board::bitboard::BitBoard;
use crate::board::moves;
use crate::board::position::Position;
use crate::board::types::*;
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

    pub fn apply_move(&mut self, from: u8, to: u8) -> bool {
        if (move_gen::gen_moves(&self.pos, from) & BitBoard::from_bit(to)).is_empty() {
            return false;
        }

        let m = match moves::create_move(&self.pos, from, to) {
            Some(m) => m,
            None => return false,
        };

        moves::do_move(&mut self.pos, &m);
        self.history.push(m);

        true
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some(last_move) = self.history.pop() {
            moves::undo_move(&mut self.pos, &last_move);
            return true;
        }

        false
    }

    pub fn apply_move_str(&mut self, input: &str) -> bool {
        if let Some((from, to)) = parse_move(input) { self.apply_move(from, to) } else { false }
    }

    pub fn gen_moves(&self, square: u8) -> u64 {
        move_gen::gen_moves(&self.pos, square).get()
    }
}

fn parse_move(input: &str) -> Option<(u8, u8)> {
    if input.len() != 4 {
        return None;
    }

    let from_file = input.chars().nth(0)? as u8 - b'a';
    let from_rank = input.chars().nth(1)? as u8 - b'1';
    let to_file = input.chars().nth(2)? as u8 - b'a';
    let to_rank = input.chars().nth(3)? as u8 - b'1';

    if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
        return None;
    }

    Some((make_square(from_file, from_rank), make_square(to_file, to_rank)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
