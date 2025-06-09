// Build: wasm-pack build --target web
pub mod board;
pub mod moves;
pub mod types;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn create_board() -> String {
    let board = board::Board::new();
    board.to_string()
}

#[wasm_bindgen]
pub struct Engine {
    board: board::Board,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            board: board::Board::new(),
        }
    }

    pub fn to_string(&self) -> String {
        self.board.to_string()
    }

    pub fn pretty_string(&self) -> String {
        self.board.pretty_string()
    }

    pub fn parse_fen(&mut self, fen: &str) -> Result<(), JsValue> {
        match self.board.parse_fen(fen) {
            Ok(()) => Ok(()),
            Err(err) => Err(JsValue::from_str(&err)),
        }
    }

    pub fn gen_moves(&self, col: u8, row: u8) -> u64 {
        let file = unsafe { std::mem::transmute(col) };
        let rank = unsafe { std::mem::transmute(7 - row) };
        moves::gen_moves(&self.board, file, rank)
    }
}
