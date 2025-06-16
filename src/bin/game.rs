use std::io::{self, Write};

use bitboard_x::engine::*;
use bitboard_x::game::player::*;
use bitboard_x::game::*;

fn main() {
    let mut game = Game::new();

    loop {
        println!("{}", utils::debug_string(game.pos()));

        if game.game_over() {
            println!("Game over!");
            break;
        }

        'mainloop: loop {
            print!("Please enter your move (e.g., e2e4):");
            io::stdout().flush().unwrap();
            let action = {
                let fen = game.fen();
                let active_player = game.active_player();
                active_player.poll_move(fen)
            };

            match action {
                PlayerAction::Pending => {
                    continue;
                }
                PlayerAction::Ready(mv) => {
                    if game.execute(&mv) {
                        break;
                    } else {
                        println!("Invalid move: {}", mv);
                    }
                }
                PlayerAction::Error(_err) => {
                    break 'mainloop;
                }
            }
        }
    }
}
