// use super::Game;
use crate::engine::board::Move;
use std::io::{self, Write};

pub trait Player {
    fn request_move(&mut self);
    fn poll_move(&mut self) -> Option<String>;
}

pub struct WebPlayer {
    selected_move: Option<Move>,
}

impl WebPlayer {
    pub fn inject_move(&mut self, mv: Move) {
        self.selected_move = Some(mv);
    }
}

impl Player for WebPlayer {
    fn request_move(&mut self) {
        // Show UI prompt, JS side will call inject_move
    }

    fn poll_move(&mut self) -> Option<String> {
        // self.selected_move.take()
        None
    }
}

pub struct ConsolePlayer;

impl Player for ConsolePlayer {
    fn request_move(&mut self) {
        print!("Please enter your move (e.g., e2e4):");
        io::stdout().flush().unwrap();
    }

    fn poll_move(&mut self) -> Option<String> {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
            }
            Err(_) => None,
        }
    }
}
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
