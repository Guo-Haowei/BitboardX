// use bitboard_x::core::utils::debug_string;
use bitboard_x::game::ai_player::AiPlayer;
use bitboard_x::game::*;

use bitboard_x::core::position::Position;
use bitboard_x::core::types::*;
use std::io::{self, Write};

fn print_board(out: &mut io::Stdout, pos: &Position) {
    const SEP: &str = " +---+---+---+---+---+---+---+---+";
    write!(out, "{}\n", SEP).unwrap();
    for rank in (0..8).rev() {
        write!(out, " | ").unwrap();
        for file in 0..8 {
            let sq = Square::make(File(file), Rank(rank));
            let piece = pos.get_piece_at(sq).to_char();
            write!(out, "{} | ", if piece == '.' { ' ' } else { piece }).unwrap();
        }

        write!(out, "{} \n{}\n", rank + 1, SEP).unwrap();
    }
    write!(out, "   a   b   c   d   e   f   g   h\n").unwrap();
    write!(out, "\nFen: {}\n\n", pos.fen()).unwrap();
}

fn main() {
    unsafe {
        use std::env;
        env::set_var("RUST_BACKTRACE", "1");
    };

    // let fen = "7k/2P5/1P6/8/8/8/8/K7 w - - 0 1";
    let mut game = GameState::new();

    let player: Box<dyn Player> = Box::new(AiPlayer::new());
    let player = Box::new(ConsolePlayer);
    game.set_white(player);

    let player: Box<dyn Player> = Box::new(AiPlayer::new());
    game.set_black(player);

    'mainloop: loop {
        let mut io = io::stdout();
        print_board(&mut io, &game.pos());

        if game.game_over() {
            println!("Game over!");
            break;
        }

        loop {
            let action = {
                let commands = game.pos_and_moves.clone();
                let active_player = game.active_player();
                active_player.request_move();
                active_player.poll_move(&commands)
            };

            match action {
                PlayerAction::Pending => {
                    continue;
                }
                PlayerAction::Ready(mv) => {
                    if game.execute(&mv).is_some() {
                        break;
                    }
                    println!("Invalid move: {}", mv);
                }
                PlayerAction::Error(err) => {
                    println!("Error occurred: {}", err);
                    break 'mainloop;
                }
            }
        }
    }
}
