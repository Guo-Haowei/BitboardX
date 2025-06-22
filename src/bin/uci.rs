fn main() {
    use bitboard_x::engine::Engine;
    use bitboard_x::utils::*;
    use std::io::{self, BufRead};

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
