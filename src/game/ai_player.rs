use super::player::{Player, PlayerAction};
use crate::engine::{move_gen, position::Position};

use rand::Rng;

pub struct AiPlayer;

impl Player for AiPlayer {
    fn request_move(&mut self) {}

    fn poll_move(&mut self, fen: String) -> PlayerAction {
        match Position::from(fen.as_str()) {
            Ok(pos) => {
                let moves = move_gen::legal_moves(&pos);
                let mut rng = rand::thread_rng();

                let n: u32 = rng.gen_range(0u32..moves.count() as u32);
                let m = moves.get(n as usize).unwrap().to_string();

                PlayerAction::Ready(m)
            }
            Err(_) => PlayerAction::Error("Invalid FEN string".to_string()),
        }
    }
}
