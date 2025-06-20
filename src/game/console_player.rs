use std::any::Any;
use std::io::{self, Write};

use super::player::{Player, PlayerAction};

pub struct ConsolePlayer;

impl Player for ConsolePlayer {
    fn name(&self) -> String {
        "ConsolePlayer".to_string()
    }

    fn request_move(&mut self) {
        print!(">> Please enter your move (e.g., e2e4):");
        io::stdout().flush().unwrap();
    }

    fn poll_move(&mut self, _: &String) -> PlayerAction {
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
