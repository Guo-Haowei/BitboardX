// use std::io::{self, Write};

use bitboard_x::engine::utils::debug_string;
use bitboard_x::game::ai_player::AiPlayer;
use bitboard_x::game::*;

// fn print_board(out: &mut io::Stdout, pos: &Position) {
//     const SEP: &str = " +---+---+---+---+---+---+---+---+";
//     write!(out, "{}\n", SEP).unwrap();
//     for rank in (0..8).rev() {
//         write!(out, " | ").unwrap();
//         for file in 0..8 {
//             let sq = Square::make(file, rank);
//             let piece = pos.get_piece_at(sq).to_char();
//             write!(out, "{} | ", if piece == '.' { ' ' } else { piece }).unwrap();
//         }

//         write!(out, "{} \n{}\n", rank + 1, SEP).unwrap();
//     }
//     write!(out, "   a   b   c   d   e   f   g   h\n").unwrap();
//     write!(out, "\nFen: {}\n\n", pos.fen()).unwrap();
// }

fn main() {
    let mut game = GameState::new();

    let player: Box<dyn Player> = Box::new(AiPlayer::new());
    game.set_white(player);

    let player: Box<dyn Player> = Box::new(AiPlayer::new());
    game.set_white(player);

    'mainloop: loop {
        println!("{}", debug_string(game.pos()));

        if game.game_over() {
            println!("Game over!");
            break;
        }

        loop {
            let action = {
                let fen = game.fen();
                let active_player = game.active_player();
                active_player.request_move();
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
                PlayerAction::Error(err) => {
                    println!("Error occurred: {}", err);
                    break 'mainloop;
                }
            }
        }
    }
}
