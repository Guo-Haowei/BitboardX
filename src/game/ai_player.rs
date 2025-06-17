use std::any::Any;

use super::player::{Player, PlayerAction};
use crate::ai::search;
use crate::core::position::Position;

pub struct AiPlayer;

impl Player for AiPlayer {
    fn name(&self) -> String {
        "AiPlayer".to_string()
    }

    fn request_move(&mut self) {}

    fn poll_move(&mut self, fen: String) -> PlayerAction {
        match Position::from_fen(fen.as_str()) {
            Ok(pos) => {
                let mut pos = pos;
                let mv = search(&mut pos, 5).unwrap();
                let mv = mv.to_string();

                PlayerAction::Ready(mv)
            }
            Err(_) => PlayerAction::Error("Invalid FEN string".to_string()),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AiPlayer {
    pub fn new() -> Self {
        Self {}
    }
}
