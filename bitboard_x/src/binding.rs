use wasm_bindgen::prelude::*;

use crate::core::move_gen::*;
use crate::core::{position::*, types::*};
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
}

#[wasm_bindgen]
impl WasmMove {
    pub fn to_string(&self) -> String {
        self.mv.to_string()
    }
}

// ---------------------------- Position Binding -------------------------------
#[wasm_bindgen]
pub struct WasmPosition {
    pos: Position,
    legal_moves: MoveList,

    undo_stack: Vec<(Move, UndoState)>,
    redo_stack: Vec<(Move, UndoState)>,
}

#[wasm_bindgen]
impl WasmPosition {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let pos = match Position::from_fen(fen) {
            Ok(pos) => pos,
            Err(_) => Position::new(),
        };

        let legal_moves = legal_moves(&pos);
        Self { pos, legal_moves, undo_stack: Vec::new(), redo_stack: Vec::new() }
    }

    pub fn fen(&self) -> String {
        self.pos.fen()
    }

    pub fn debug_string(&self) -> String {
        utils::debug_string(&self.pos)
    }

    pub fn legal_moves(&self) -> Vec<String> {
        self.legal_moves.iter().map(|mv| mv.to_string()).collect()
    }

    pub fn turn(&self) -> u8 {
        self.pos.side_to_move.as_u8()
    }

    pub fn make_move(&mut self, mv_str: String) -> bool {
        let mv = utils::parse_move(mv_str.as_str());
        if mv.is_none() {
            return false;
        }

        let mv = mv.unwrap();
        let src = mv.src_sq();
        let dst = mv.dst_sq();
        let promtion = mv.get_promotion();
        let mut move_legal = false;
        for mv in self.legal_moves.iter().copied() {
            if mv.src_sq() == src && mv.dst_sq() == dst && mv.get_promotion() == promtion {
                move_legal = true;
                break;
            }
        }

        if move_legal {
            let undo_state = self.pos.make_move(mv);

            self.undo_stack.push((mv, undo_state));
            self.redo_stack.clear();

            self.legal_moves = legal_moves(&self.pos);
        }

        move_legal
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
        self.engine.uci_cmd_position(args);
    }

    pub fn best_move(&mut self, depth: u8) -> Option<WasmMove> {
        match self.engine.best_move(depth) {
            Some(mv) => Some(WasmMove { mv }),
            None => None,
        }
    }
}
