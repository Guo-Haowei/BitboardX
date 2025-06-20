use rustyline::{DefaultEditor, Result};

use bitboard_x::engine::{self, Engine};

fn main() -> Result<()> {
    use std::io::{self};

    eprintln!("{}", Engine::name());
    let mut stdout = io::stdout();
    let mut engine = Engine::new();
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

                if !engine.handle_uci_cmd(&mut stdout, input) {
                    break;
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
