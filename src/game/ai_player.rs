use std::any::Any;

use super::player::{Player, PlayerAction};
use crate::engine::Engine;

pub struct AiPlayer {
    engine: Engine,
}

impl Player for AiPlayer {
    fn name(&self) -> String {
        "AiPlayer".to_string()
    }

    fn request_move(&mut self) {}

    fn poll_move(&mut self, cmd: &String) -> PlayerAction {
        use std::io::{self};

        let out = &mut io::sink();

        self.engine.handle_uci_cmd(out, &cmd.as_str());

        let mv = self.engine.best_move(6).unwrap();
        let mv = mv.to_string();

        PlayerAction::Ready(mv)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AiPlayer {
    pub fn new() -> Self {
        Self { engine: Engine::new() }
    }
}
