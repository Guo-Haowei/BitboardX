use rustyline::{DefaultEditor, Result};
fn main() -> Result<()> {
    use bitboard_x::engine::Engine;
    use bitboard_x::utils::*;
    use std::io::{self};
    logger::init_logger();

    eprintln!("{}", Engine::name());
    let mut stdout = io::stdout();
    let mut engine = Engine::new();
    let mut rl = DefaultEditor::new()?;
    // let _ = rl.load_history("history.txt").is_err();

    // @TODO: take out UCI logic from the engine and put it in a separate module
    // @TODO: get rid of rustyline dependency, use std::io instead
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
