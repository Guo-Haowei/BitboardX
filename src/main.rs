pub mod engine;
pub mod game;
mod uci;

use rustyline::Result;
use std::env;
use uci::*;

// fn game_main() {
//     let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//     let mut _game = game::Game::new(fen);

//     loop {
//         let board = _game.to_string(true);
//         println!("{}------", board);

//         loop {
//             print!("Enter move (e.g. e2e4): ");
//             io::stdout().flush().unwrap();

//             let mut input = String::new();
//             if io::stdin().read_line(&mut input).is_err() {
//                 println!("Error reading input.");
//                 continue;
//             }

//             let input = input.trim();

//             if input == "quit" {
//                 return;
//             }

//             // if game.execute(input) {
//             //     break;
//             // }

//             println!("Invalid move: {}", input);
//             println!("------");
//             panic!("Game execution not implemented yet");
//         }
//     }
// }

fn print_usage() {
    println!("Usage: {} [--help] [--version] [--game]", NAME);
    println!("Options:");
    println!("  --help     Show this help message");
    println!("  --version  Show version information");
    println!("  --game     Start a game session");
}

fn print_version() {
    println!("{}", name());
}

fn main() -> Result<()> {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();
    if argc == 1 {
        return uci::uci_main();
    }

    let command = argv[1].as_str();
    match command {
        "--help" | "-h" => {
            print_usage();
        }
        "--version" | "-v" => {
            print_version();
        }
        // "--game" | "-g" => {
        //     game_main();
        // }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }
    Ok(())
}
