use crate::core::board::Position;
use crate::core::types::make_square;
use crate::engine::move_gen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    pos: Position,
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
        Self { pos: position }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.pos.state.to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.pos.state.to_board_string()
    }

    pub fn apply_move(&mut self, from: u8, to: u8) -> bool {
        if move_gen::gen_moves(&self.pos, from) & (1u64 << to) == 0 {
            return false;
        }
        self.pos.apply_move(from, to)
    }

    pub fn apply_move_str(&mut self, input: &str) -> bool {
        if let Some((from, to)) = parse_move(input) { self.apply_move(from, to) } else { false }
    }

    pub fn gen_moves(&self, square: u8) -> u64 {
        move_gen::gen_moves(&self.pos, square)
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
