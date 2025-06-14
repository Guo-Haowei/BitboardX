use crate::engine::board::Move;
use crate::engine::position::Position;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MoveJs {
    internal: Move,
}

#[wasm_bindgen]
impl MoveJs {
    fn new(m: &Move) -> Self {
        Self { internal: m.clone() }
    }

    pub fn from_sq(&self) -> u8 {
        self.internal.from_sq().as_u8()
    }

    pub fn to(&self) -> u8 {
        self.internal.to_sq().as_u8()
    }
}

#[wasm_bindgen]
pub struct Game {
    pos: Position,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let pos = match Position::from(fen) {
            Ok(pos) => pos,
            Err(err) => {
                eprintln!("Error parsing FEN: {}", err);
                Position::new()
            }
        };

        Self { pos }
    }

    pub fn to_string(&self, pad: bool) -> String {
        self.pos.to_string(pad)
    }

    pub fn to_board_string(&self) -> String {
        self.pos.to_board_string()
    }

    pub fn can_undo(&self) -> bool {
        self.pos.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.pos.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        self.pos.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.pos.redo()
    }

    pub fn execute(&mut self, m: MoveJs) -> bool {
        self.pos.do_move(&m.internal);
        true
    }

    pub fn legal_moves(&mut self) -> Vec<MoveJs> {
        let mut res = Vec::new();
        let moves = self.pos.legal_moves();

        for m in moves {
            res.push(MoveJs::new(&m));
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            game.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_apply_move() {
        // let mut game = Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        // assert!(game.execute("e2e4")); // e2 to e4
        // assert_eq!(
        //     game.to_board_string(),
        //     "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR"
        // );
        // game.undo();
        // assert_eq!(
        //     game.to_board_string(),
        //     "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        // );
        // game.redo();
        // assert_eq!(
        //     game.to_board_string(),
        //     "rnbqkbnrpppppppp....................P...........PPPP.PPPRNBQKBNR"
        // );

        // panic!(
        //     "Test failed, this is a placeholder panic to indicate the test should be implemented."
        // );
    }
}
