fn main() {
    use bitboard_x::engine::Engine;
    use bitboard_x::utils::*;
    use std::io::{self, BufRead};

    unsafe {
        use std::env;
        env::set_var("RUST_BACKTRACE", "1");
    };

    logger::init_logger();

    let stdin = io::stdin();
    let reader = stdin.lock();

    let mut stdout = io::stdout();
    let mut engine = Engine::new();

    eprintln!("{}", Engine::name());

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                if !engine.handle_uci_cmd(&mut stdout, line.trim()) {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }
}

// fn print_board(out: &mut io::Stdout, pos: &Position) {
//     const SEP: &str = " +---+---+---+---+---+---+---+---+";
//     write!(out, "{}\n", SEP).unwrap();
//     for rank in (0..8).rev() {
//         write!(out, " | ").unwrap();
//         for file in 0..8 {
//             let sq = Square::make(File(file), Rank(rank));
//             let piece = pos.get_piece_at(sq).to_char();
//             write!(out, "{} | ", if piece == '.' { ' ' } else { piece }).unwrap();
//         }

//         write!(out, "{} \n{}\n", rank + 1, SEP).unwrap();
//     }
//     write!(out, "   a   b   c   d   e   f   g   h\n").unwrap();
//     write!(out, "\nFen: {}\n\n", pos.fen()).unwrap();
// }
