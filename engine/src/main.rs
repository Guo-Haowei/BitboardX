pub mod core;
pub mod engine;
pub mod game;

use std::io::{self, Write};

fn main() {
    let mut game = game::Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    loop {
        let board = game.to_string(true);
        print!("{}------\nEnter move (e.g. e2e4): ", board);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input.");
            continue;
        }

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        if !game.apply_move_str(input) {
            println!("Invalid move: {}", input);
        }

        println!("------");
    }
}
