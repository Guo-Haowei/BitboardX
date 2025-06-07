pub mod types;
pub mod board;

fn main() {
    let board = board::Board::new();
    println!("{}", board.to_string());
}
