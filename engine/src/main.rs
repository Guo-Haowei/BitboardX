pub mod board;
pub mod engine;
pub mod game;

use engine::{Engine, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH};
use std::env;
use std::io::{self, BufRead, Write};

fn game_main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut game = game::Game::new(fen);

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

            if input == "quit" {
                return;
            }

            if game.apply_move_str(input) {
                break;
            }

            println!("Invalid move: {}", input);
            println!("------");
        }
    }
}

fn uci_main() {
    eprintln!("UCI Protocol Engine: {}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH);

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut engine = Engine::new();

    // @TODO: better pattern matching for commands
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        if input == "uci" {
            engine.cmd_uci(&mut stdout);
        } else if input == "isready" {
            engine.cmd_isready(&mut stdout);
        } else if input.starts_with("position") {
            engine.cmd_position(&mut stdout, &input);
        } else if input.starts_with("go") {
            engine.cmd_go(&mut stdout);
        } else if input == "quit" {
            engine.shutdown();
            break;
        } else {
            eprintln!("Unknown command: {}", input);
        }

        stdout.flush().unwrap();
    }
}

fn print_usage() {
    println!("Usage: engine [--help] [--version] [--game]");
    println!("Options:");
    println!("  --help     Show this help message");
    println!("  --version  Show version information");
    println!("  --game     Start a game session");
}

fn print_version() {
    println!("Engine version {}.{}.{}", engine::VERSION_MAJOR, engine::VERSION_MINOR, engine::VERSION_PATCH);
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();
    if argc == 1 {
        uci_main();
        return;
    }

    let command = argv[1].as_str();
    match command {
        "--help" | "-h" => {
            print_usage();
        }
        "--version" | "-v" => {
            print_version();
        }
        "--game" | "-g" => {
            game_main();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }
}
