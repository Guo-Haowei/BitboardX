use std::io::{self, Write};

pub mod board;
pub mod fen_state;
pub mod moves;
pub mod types;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "r1bqkb1r/8/8/8/8/8/8/R1BQKB1R w KQkq - 0 1";
    let mut board = board::Board::from_fen(fen).unwrap();

    loop {
        let board_string = board.state.to_string(true);
        print!("{}------\nEnter move (e.g. e2e4): ", board_string);
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
                if board.apply_move(from, to) {
                    println!("Move applied: {} to {}", from, to);
                } else {
                    println!("Invalid move: {} to {}", from, to);
                }
            }
            None => println!("Invalid input '{}'.", input),
        }

        println!("------");
    }
}
