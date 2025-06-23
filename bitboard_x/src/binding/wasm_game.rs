use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::{
    core::types::*,
    game::{player::GuiPlayer, *},
    // logger,
    utils::*,
};

#[wasm_bindgen]
pub struct WasmMove {
    mv: Option<Move>,
}

#[wasm_bindgen]
impl WasmMove {
    pub fn is_none(&self) -> bool {
        self.mv.is_none()
    }

    pub fn src_sq(&self) -> String {
        assert!(self.mv.is_some());
        self.mv.unwrap().src_sq().to_string()
    }

    pub fn dst_sq(&self) -> String {
        assert!(self.mv.is_some());
        self.mv.unwrap().dst_sq().to_string()
    }

    pub fn is_castling(&self) -> bool {
        assert!(self.mv.is_some());
        let mv = self.mv.as_ref().unwrap();
        mv.get_type() == MoveType::Castling
    }
}

#[wasm_bindgen]
pub struct WasmGame {
    internal: GameState,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut game = Self { internal: GameState::new() };

        let player: Box<dyn Player> = Box::new(GuiPlayer::new());
        game.internal.set_white(player);

        let player: Box<dyn Player> = Box::new(AiPlayer::new());
        game.internal.set_black(player);

        game
    }

    pub fn reset_game(&mut self, fen: String, is_white_human: bool, is_black_human: bool) {
        let game = match GameState::from_fen(fen.as_str()) {
            Ok(game) => game,
            Err(err) => {
                console::error_1(&format!("Invalid FEN string: {}", err).into());
                GameState::new()
            }
        };

        self.internal = game;

        fn create_player(human: bool) -> Box<dyn Player> {
            if human { Box::new(GuiPlayer::new()) } else { Box::new(AiPlayer::new()) }
        }

        self.internal.set_white(create_player(is_white_human));
        self.internal.set_black(create_player(is_black_human));
    }

    pub fn get_move(&mut self) -> Option<String> {
        // logger::log(format!("command: {}", self.pos_and_moves).to_string());

        let action = {
            let commands = self.internal.pos_and_moves.clone();
            let player = self.internal.active_player();
            player.request_move();
            player.poll_move(&commands)
        };

        match action {
            PlayerAction::Pending => None,
            PlayerAction::Ready(mv) => Some(mv),
            PlayerAction::Error(err) => {
                console::error_1(&format!("Player error: {}", err).into());
                None
            }
        }
    }

    pub fn make_move(&mut self, mv_str: String) -> WasmMove {
        let mv = self.internal.execute(&mv_str);

        WasmMove { mv }
    }

    pub fn get_legal_moves(&self) -> Vec<String> {
        self.internal.legal_moves.iter().map(|mv| mv.to_string()).collect()
    }

    // @TODO: DONT LIKE THIS, FIND A BETTER WAY
    pub fn inject_move(&mut self, mv: String) {
        if let Some(player) = self.internal.active_player().as_any_mut().downcast_mut::<GuiPlayer>()
        {
            player.inject_move(mv);
        }
    }

    pub fn fen(&self) -> String {
        self.internal.fen()
    }

    pub fn debug_string(&self) -> String {
        debug_string(self.internal.pos())
    }

    // @TODO: game status, running, draw, white wins, black wins
    pub fn game_over(&self) -> bool {
        self.internal.game_over()
    }

    // @TODO: fix undo and redo
    pub fn can_undo(&self) -> bool {
        self.internal.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.internal.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        self.internal.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.internal.redo()
    }
}
