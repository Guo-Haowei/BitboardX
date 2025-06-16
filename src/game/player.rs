pub enum PlayerAction {
    Pending,
    Ready(String),
    Error(String),
}

pub trait Player {
    fn name(&self) -> String;
    fn request_move(&mut self);
    fn poll_move(&mut self, fen: String) -> PlayerAction;
}

pub struct NullPlayer;

impl Player for NullPlayer {
    fn name(&self) -> String {
        "NullPlayer".to_string()
    }

    fn request_move(&mut self) {}

    fn poll_move(&mut self, _fen: String) -> PlayerAction {
        panic!("NullPlayer does not support moves");
    }
}
