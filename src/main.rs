use bitboard_x::core::{position::Position, types::*};
use bitboard_x::engine::Engine;
use bitboard_x::utils::*;
use std::io::Write;
use std::io::{self, BufRead};

fn main() {
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

                let mut parts = line.trim().splitn(2, ' ');
                let cmd = parts.next().unwrap();
                let args = parts.next().unwrap_or("");

                match cmd {
                    "uci" => uci_cmd_uci(&mut stdout),
                    "ucinewgame" => uci_cmd_ucinewgame(&mut engine, &mut stdout),
                    "isready" => uci_cmd_isready(&mut stdout),
                    "position" => match engine.set_position(args) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("Error: {}", err);
                        }
                    },
                    "go" => uci_cmd_go(&mut engine, &mut stdout, args),
                    "d" => uci_cmd_d(&mut engine, &mut stdout),
                    "q" | "quit" => {
                        break;
                    }
                    _ => {
                        eprintln!("Unknown command: '{}'. Type help for more information.", line);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }
}

pub fn uci_cmd_isready<W: Write>(writer: &mut W) {
    writeln!(writer, "readyok").unwrap();
}

pub fn uci_cmd_ucinewgame<W: Write>(engine: &mut Engine, _: &mut W) {
    engine.state.set_position(Position::new());
}

pub fn uci_cmd_uci<W: Write>(writer: &mut W) {
    writeln!(writer, "id name {}", Engine::name()).unwrap();
    writeln!(writer, "id author haguo").unwrap();
    writeln!(writer, "uciok").unwrap();
}

pub fn uci_cmd_d<W: Write>(engine: &Engine, writer: &mut W) {
    print_board(writer, &engine.state.pos);
}

pub fn uci_cmd_go<W: Write>(engine: &mut Engine, writer: &mut W, args: &str) {
    let parts: Vec<&str> = args.split_whitespace().collect();

    match parts.as_slice() {
        ["perft", depth, _rest @ ..] => {
            let depth: u8 = match depth.parse() {
                Ok(d) if d <= 8 => d,
                _ => {
                    eprintln!("Error: Invalid depth '{}'. Must be between 0 and 8.", depth);
                    return;
                }
            };
            engine.perft_test(writer, depth);
        }
        ["wtime", wtime, "btime", btime, "movestogo", movestogo, _rest @ ..] => {
            let wtime: i32 = wtime.trim().parse().unwrap();
            let btime: i32 = btime.trim().parse().unwrap();
            let movestogo: i32 = movestogo.trim().parse().unwrap();

            let time = if engine.state.pos.white_to_move() { wtime } else { btime };
            let time = time as f64 / movestogo as f64;
            let time = time * 0.9;

            let mv = engine.best_move(time).unwrap();
            writeln!(writer, "bestmove {}", mv.to_string()).unwrap();
        }
        _ => panic!(
            "Error: Invalid 'go' command arguments. Expected 'perft <depth>' or 'wtime <wtime> btime <btime> movestogo <movestogo>'."
        ),
    }
}

fn print_board<W: Write>(out: &mut W, pos: &Position) {
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
