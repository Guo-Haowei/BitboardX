use std::any::Any;

pub enum PlayerAction {
    Pending,
    Ready(String),
    Error(String),
}

pub trait Player: Any {
    fn name(&self) -> String;
    fn request_move(&mut self);
    fn poll_move(&mut self, commands: &String) -> PlayerAction;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct NullPlayer;

impl Player for NullPlayer {
    fn name(&self) -> String {
        "NullPlayer".to_string()
    }

    fn request_move(&mut self) {}

    fn poll_move(&mut self, _fen: &String) -> PlayerAction {
        panic!("NullPlayer does not support moves");
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct GuiPlayer {
    buffered_move: Option<String>,
}

impl Player for GuiPlayer {
    fn name(&self) -> String {
        "GuiPlayer".to_string()
    }

    fn request_move(&mut self) {
        // GUI-specific logic to request a move
    }

    fn poll_move(&mut self, _: &String) -> PlayerAction {
        if let Some(mv) = self.buffered_move.take() {
            return PlayerAction::Ready(mv);
        }
        PlayerAction::Pending
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl GuiPlayer {
    pub fn new() -> Self {
        Self { buffered_move: None }
    }

    pub fn inject_move(&mut self, mv: String) {
        self.buffered_move = Some(mv);
    }
}
