// use super::Game;
use std::io::{self};

pub enum PlayerAction {
    Pending,
    Ready(String),
    Error(String),
}

pub trait Player {
    fn request_move(&mut self);
    fn poll_move(&mut self, fen: String) -> PlayerAction;
}

pub struct ConsolePlayer;

impl Player for ConsolePlayer {
    fn request_move(&mut self) {}

    fn poll_move(&mut self, _pos: String) -> PlayerAction {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    PlayerAction::Pending
                } else {
                    PlayerAction::Ready(trimmed.to_string())
                }
            }
            Err(_) => PlayerAction::Error("Failed to read input".to_string()),
        }
    }
}

// pub struct WebPlayer {
//     selected_move: Option<Move>,
// }

// impl WebPlayer {
//     pub fn inject_move(&mut self, mv: Move) {
//         self.selected_move = Some(mv);
//     }
// }

// impl Player for WebPlayer {
//     fn request_move(&mut self) {
//         // Show UI prompt, JS side will call inject_move
//     }

//     fn poll_move(&mut self) -> PlayerAction {
//         // self.selected_move.take()
//         PlayerAction::Pending
//     }
// }

// pub struct AiPlayer;

// impl Player for AiPlayer {
//     fn request_move(&mut self, game: &Game) {
//         self.next_move = Some(minimax(game.pos.clone()));
//     }

//     fn poll_move(&mut self) -> Option<Move> {
//         self.next_move.take()
//     }
// }

// pub struct RemotePlayer {
//     request_sent: bool,
//     move_buffer: Option<Move>,
// }

// impl Player for RemotePlayer {
//     fn request_move(&mut self, game: &Game) {
//         if !self.request_sent {
//             send_http(game.pos.clone()); // async network
//             self.request_sent = true;
//         }
//     }

//     fn poll_move(&mut self) -> Option<Move> {
//         check_response() // returns Option<Move>
//     }
// }
