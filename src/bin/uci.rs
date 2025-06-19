use rustyline::{DefaultEditor, Result};

use bitboard_x::core::name;
use bitboard_x::uci::UCI;

fn main() -> Result<()> {
    use std::io::{self};

    eprintln!("{}", name());
    let mut stdout = io::stdout();
    let mut uci = UCI::new();
    let mut rl = DefaultEditor::new()?;
    // let _ = rl.load_history("history.txt").is_err();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let input = line.trim();

                if !input.is_empty() {
                    let _ = rl.add_history_entry(line.as_str());
                }

                let mut parts = input.splitn(2, ' ');
                let cmd = parts.next().unwrap();
                let args = parts.next().unwrap_or("");

                match cmd {
                    "uci" => uci.command_uci(&mut stdout),
                    "ucinewgame" => uci.command_ucinewgame(&mut stdout),
                    "isready" => uci.command_isready(&mut stdout),
                    "position" => uci.command_position(&mut stdout, args),
                    "go" => uci.command_go(&mut stdout, args),
                    "d" => uci.command_d(&mut stdout),
                    "q" | "quit" => {
                        uci.shutdown();
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

    // rl.save_history("history.txt")?;
    Ok(())
}
