pub mod engine;
pub mod game;

use engine::board::move_::Move;
use engine::engine::*;
use engine::position::Position;
use rustyline::{DefaultEditor, Result};
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

            panic!("Game execution not implemented yet");
            // if game.execute(input) {
            //     break;
            // }

            println!("Invalid move: {}", input);
            println!("------");
        }
    }
}

fn uci_main() -> Result<()> {
    println!("{}", name());
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
            _ => {
                break;
            }
        }
    }

    Ok(())
}

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

fn perft_main() {
    for depth in 1..=8 {
        let mut pos = engine::position::Position::new();
        let nodes = perft_test(&mut pos, depth);
        println!("Depth {}: {} nodes", depth, nodes);
    }
}

fn perft_test(pos: &mut Position, depth: u8) -> u64 {
    let mut nodes = 0u64;

    let move_list = pos.legal_moves();

    if depth == 1 {
        return move_list.len() as u64;
    }

    for m in move_list {
        let snapshot = pos.make_move(&m);
        nodes += perft_test(pos, depth - 1);
        pos.unmake_move(&m, &snapshot);
    }

    nodes
}

fn main() -> Result<()> {
    if true {
        perft_main();
        return Ok(());
    }

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
