use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::{engine::utils, game::*};

#[wasm_bindgen]
pub struct WasmGameState {
    internal: GameState,
}

#[wasm_bindgen]
impl WasmGameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut game = Self { internal: GameState::new() };

        game.internal.set_white(Box::new(AiPlayer));
        game.internal.set_black(Box::new(AiPlayer));

        game
    }

    pub fn tick(&mut self) {
        let fen = self.internal.fen();
        let action = {
            let player = self.internal.active_player();
            player.request_move();
            player.poll_move(fen)
        };

        match action {
            PlayerAction::Pending => {}
            PlayerAction::Ready(mv) => {
                if self.internal.execute(&mv) {
                    console::log_1(&format!("Move executed: {}", mv).into());
                }
            }
            PlayerAction::Error(err) => {
                console::error_1(&format!("Player error: {}", err).into());
            }
        }
    }

    // @TODO: fen?
    pub fn to_board_string(&self) -> String {
        utils::to_board_string(self.internal.pos())
    }

    pub fn game_over(&self) -> bool {
        self.internal.game_over()
    }

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
