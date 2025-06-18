use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::{
    core::utils,
    game::{player::GuiPlayer, *},
};

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
        console::log_1(&format!("Resetting game with FEN: {}", fen).into());
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
        let fen = self.internal.fen();
        let action = {
            let player = self.internal.active_player();
            player.request_move();
            player.poll_move(fen)
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

    pub fn make_move(&mut self, mv: String) -> bool {
        if !self.internal.execute(&mv) {
            // console::error_1(&format!("Invalid move by {}: {}", name, mv).into());
            return false;
        }

        let name = self.internal.active_player().name();
        console::log_1(&format!("Player {} => {}", name, mv).into());
        true
    }

    pub fn get_legal_moves(&self) -> Vec<String> {
        self.internal.legal_moves.iter().map(|m| m.to_string()).collect()
    }

    pub fn inject_move(&mut self, mv: String) {
        if let Some(player) = self.internal.active_player().as_any_mut().downcast_mut::<GuiPlayer>()
        {
            player.inject_move(mv);
        }
    }

    pub fn debug_string(&self) -> String {
        utils::debug_string(self.internal.pos())
    }

    pub fn to_board_string(&self) -> String {
        utils::to_board_string(self.internal.pos())
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
