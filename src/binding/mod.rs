use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::{
    core::utils,
    game::{player::GuiPlayer, *},
};

#[wasm_bindgen]
pub struct WasmGameState {
    internal: GameState,
}

#[wasm_bindgen]
impl WasmGameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // let fen = "r4r1k/2p1p2p/p5p1/1p1Q1p2/1P3bq1/P1P2N2/1B3P2/4R1RK w - - 0 1";
        let mut game = Self { internal: GameState::new() };

        let player: Box<dyn Player> = Box::new(GuiPlayer::new());
        game.internal.set_white(player);

        let player: Box<dyn Player> = Box::new(AiPlayer::new());
        game.internal.set_black(player);

        game
    }

    pub fn reset_game(&mut self, fen: String, white_player_human: bool, black_player_human: bool) {
        console::log_1(&format!("Resetting game with FEN: {}", fen).into());
        let game = match GameState::from_fen(fen.as_str()) {
            Ok(game) => game,
            Err(err) => {
                console::error_1(&format!("Invalid FEN string: {}", err).into());
                GameState::new()
            }
        };

        self.internal = game;

        if white_player_human {
            let player: Box<dyn Player> = Box::new(GuiPlayer::new());
            self.internal.set_white(player);
        } else {
            let player: Box<dyn Player> = Box::new(AiPlayer::new());
            self.internal.set_white(player);
        }

        if black_player_human {
            let player: Box<dyn Player> = Box::new(GuiPlayer::new());
            self.internal.set_black(player);
        } else {
            let player: Box<dyn Player> = Box::new(AiPlayer::new());
            self.internal.set_black(player);
        }
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

    pub fn inject_move(&mut self, mv: String) {
        if let Some(player) = self.internal.active_player().as_any_mut().downcast_mut::<GuiPlayer>()
        {
            console::log_1(&format!("Injected move: {}", &mv).into());
            player.inject_move(mv);
        }
    }

    pub fn debug_string(&self) -> String {
        utils::debug_string(self.internal.pos())
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
