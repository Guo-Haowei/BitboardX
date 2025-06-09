use std::io::{self, Write};

pub mod board;
pub mod moves;
pub mod types;

fn main() {
    let mut board = board::Board::new();
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    match board.parse_fen(fen) {
        Ok(()) => {}
        Err(err) => panic!("Error parsing fen '{}', {}", fen, err),
    }

    loop {
        print!("{}", board.pretty_string());
        print!("------\nEnter move (e.g. e2e4): ");
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

        match moves::parse_move(input) {
            Some((from, to)) => {
                println!("Moving from {} to {}", from, to);
            }
            None => println!("Invalid input '{}'.", input),
        }

        println!("------");
    }
}
