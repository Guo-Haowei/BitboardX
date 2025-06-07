// Build: wasm-pack build --target web
pub mod board;
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
}