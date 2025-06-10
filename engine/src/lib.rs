// Build: wasm-pack build --target web
mod core;
mod engine;

use core::position::Position;
use engine::move_gen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ChessEngine {
    position: Position,
}

#[wasm_bindgen]
impl ChessEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { position: Position::new() }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.position.state.to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.position.state.to_board_string()
    }

    // pub fn parse_fen(&mut self, fen: &str) -> Result<(), JsValue> {
    //     match self.board.parse_fen(fen) {
    //         Ok(()) => Ok(()),
    //         Err(err) => Err(JsValue::from_str(&err)),
    //     }
    // }

    pub fn gen_moves(&self, square: u8) -> u64 {
        move_gen::gen_moves(&self.position, square)
    }

    pub fn apply_move(&mut self, from: u8, to: u8) -> bool {
        self.position.apply_move(from, to)
    }
}
