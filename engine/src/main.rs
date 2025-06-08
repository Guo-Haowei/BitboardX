pub mod board;
pub mod moves;
pub mod types;

fn main() {
    let mut board = board::Board::new();
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    match board.parse_fen(fen) {
        Ok(()) => {
            let board_string = board.pretty_string();
            println!("{}", board_string);
        },
        Err(err) => {
            println!("Error parsing fen '{}', {}", fen, err);
        }
    }
}
