pub mod board;
pub mod engine;
pub mod game;

use engine::Engine;
use rustyline::{DefaultEditor, Result, error::ReadlineError};
use std::env;
use std::io::{self, Write};

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

fn uci_main() -> Result<()> {
    eprintln!("UCI Protocol Engine: {}", engine::version());
    let mut stdout = io::stdout();
    let mut engine = Engine::new();
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                let mut parts = input.splitn(2, ' ');
                let cmd = parts.next().unwrap();
                let args = parts.next().unwrap_or("");

                rl.add_history_entry(line.as_str())?;

                match cmd {
                    "uci" => engine.cmd_uci(&mut stdout),
                    "isready" => engine.cmd_isready(&mut stdout),
                    "position" => engine.cmd_position(&mut stdout, args),
                    "go" => engine.cmd_go(&mut stdout, args),
                    "exit" | "quit" => {
                        engine.shutdown();
                        break;
                    }
                    _ => eprintln!("Unknown command: '{}'. Type help for more information.", input),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: engine [--help] [--version] [--game]");
    println!("Options:");
    println!("  --help     Show this help message");
    println!("  --version  Show version information");
    println!("  --game     Start a game session");
}

fn print_version() {
    println!("Engine version {}", engine::version());
}

fn main() -> Result<()> {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();
    if argc == 1 {
        return uci_main();
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
    Ok(())
}
