// build wasm: wasm-pack build --target web
use wasm_bindgen::prelude::*;

use crate::core::{game_state::GameState, move_gen::*, position::*, types::*};
use crate::engine::Engine;
use crate::utils;

#[wasm_bindgen]
pub fn name() -> String {
    Engine::name()
}

// ------------------------------ Move Binding ---------------------------------

#[wasm_bindgen]
pub struct WasmMove {
    mv: Move,
    mv_string: String,
    captured: char,
}

#[wasm_bindgen]
impl WasmMove {
    fn new(mv: Move, captured: Piece) -> Self {
        Self {
            mv,
            mv_string: mv.to_string(),
            captured: match captured {
                Piece::NONE => '.',
                _ => captured.to_char(),
            },
        }
    }

    pub fn src_sq(&self) -> String {
        self.mv_string[0..2].to_string()
    }

    pub fn dst_sq(&self) -> String {
        self.mv_string[2..4].to_string()
    }

    pub fn get_promotion(&self) -> char {
        if self.mv_string.len() == 5 { self.mv_string.chars().nth(4).unwrap() } else { '.' }
    }

    pub fn get_captured(&self) -> char {
        self.captured
    }

    pub fn get_type(&self) -> u8 {
        self.mv.get_type() as u8
    }

    pub fn is_castling(&self) -> bool {
        self.mv.get_type() == MoveType::Castling
    }

    pub fn is_en_passant(&self) -> bool {
        self.mv.get_type() == MoveType::EnPassant
    }

    pub fn is_promotion(&self) -> bool {
        self.mv.get_type() == MoveType::Promotion
    }

    pub fn to_string(&self) -> String {
        self.mv.to_string()
    }
}

// ---------------------------- Position Binding -------------------------------
#[wasm_bindgen]
pub struct WasmGame {
    state: GameState,
    legal_moves: MoveList,

    undo_stack: Vec<(Move, UndoState)>,
    redo_stack: Vec<(Move, UndoState)>,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let state = GameState::from_fen(fen).unwrap_or_else(|_| GameState::new());

        let legal_moves = legal_moves(&state.pos);
        Self { state, legal_moves, undo_stack: Vec::new(), redo_stack: Vec::new() }
    }

    pub fn fen(&self) -> String {
        self.state.pos.fen()
    }

    pub fn debug_string(&self) -> String {
        utils::debug_string(&self.state.pos)
    }

    pub fn board_string(&self) -> String {
        utils::board_string(&self.state.pos)
    }

    pub fn legal_moves(&self) -> Vec<String> {
        self.legal_moves.iter().map(|mv| mv.to_string()).collect()
    }

    pub fn turn(&self) -> u8 {
        self.state.pos.side_to_move.as_u8()
    }

    pub fn make_move(&mut self, mv_str: String) -> Option<WasmMove> {
        let mv = utils::parse_move(mv_str.as_str());
        if mv.is_none() {
            return None;
        }

        let mv = mv.unwrap();
        let src = mv.src_sq();
        let dst = mv.dst_sq();
        let promtion = mv.get_promotion();

        let mut final_mv: Option<WasmMove> = None;

        for mv in self.legal_moves.iter().copied() {
            if mv.src_sq() == src && mv.dst_sq() == dst && mv.get_promotion() == promtion {
                let undo_state = self.state.make_move(mv);

                self.undo_stack.push((mv, undo_state));
                self.redo_stack.clear();

                final_mv = Some(WasmMove::new(mv, self.state.pos.state.captured_piece));
                break;
            }
        }

        if !final_mv.is_none() {
            self.legal_moves = legal_moves(&self.state.pos);
        }
        final_mv
    }

    pub fn get_game_status(&self) -> String {
        if self.legal_moves.len() == 0 {
            return if self.state.pos.is_in_check(self.state.pos.side_to_move) {
                if self.state.pos.white_to_move() { "black wins" } else { "white wins" }
            } else {
                "stalemate"
            }
            .into();
        }
        if self.state.is_fifty_draw() {
            return "fifty".into();
        }
        if self.state.is_three_fold() {
            return "threefold".into();
        }

        return "playing".into();
    }
}

// --------------------------- Engine Binding ----------------------------------
#[wasm_bindgen]
pub struct WasmEngine {
    engine: Engine,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { engine: Engine::new() }
    }

    pub fn set_position(&mut self, args: &str) {
        use std::io;
        let mut null = io::sink();
        self.engine.handle_uci_cmd(&mut null, args);
    }

    pub fn best_move(&mut self, time: f64) -> String {
        match self.engine.best_move(time) {
            Some(mv) => mv.to_string(),
            None => "".to_string(),
        }
    }
}
