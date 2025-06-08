pub mod types;
pub mod board;

fn main() {
    let mut board = board::Board::new();
    let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";

    match board.parse_fen(fen) {
        Ok(()) => {
            println!("{}", board.to_string());
        },
        Err(err) => {
            println!("Error parsing fen '{}', {}", fen, err);
        }
    }
}
