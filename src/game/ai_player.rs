use super::player::{Player, PlayerAction};
use crate::engine::{move_gen, position::Position};
use std::any::Any;

pub struct AiPlayer;

impl Player for AiPlayer {
    fn name(&self) -> String {
        "AiPlayer".to_string()
    }

    fn request_move(&mut self) {}

    fn poll_move(&mut self, fen: String) -> PlayerAction {
        match Position::from(fen.as_str()) {
            Ok(pos) => {
                let moves = move_gen::legal_moves(&pos);

                let n = 0;
                let m = moves.get(n as usize).unwrap().to_string();

                PlayerAction::Ready(m)
            }
            Err(_) => PlayerAction::Error("Invalid FEN string".to_string()),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
