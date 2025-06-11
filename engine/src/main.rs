pub mod board;
pub mod engine;
pub mod game;

use std::io::{self, Write};

fn main() {
    // @TODO: basic UCI command
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "r3k2r/ppp1bppp/2n1pn2/3p4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b kq - 0 10";
    let mut game = game::Game::new(fen);
    // game.gen_moves(board::types::SQ_E8);

    loop {
        let board = game.to_string(true);
        println!("{}------", board);

        loop {
            print!("Enter move (e.g. e2e4): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("Error reading input.");
                continue;
            }

            let input = input.trim();

            if game.apply_move_str(input) {
                break;
            }

            println!("Invalid move: {}", input);
            println!("------");
        }
    }
}
