// use super::Game;

pub enum PlayerAction {
    Pending,
    Ready(String),
    Error(String),
}

pub trait Player {
    fn request_move(&mut self);
    fn poll_move(&mut self, fen: String) -> PlayerAction;
}

pub struct NullPlayer;

impl Player for NullPlayer {
    fn request_move(&mut self) {}

    fn poll_move(&mut self, _fen: String) -> PlayerAction {
        panic!("NullPlayer does not support moves");
    }
}

// pub struct RemotePlayer {
//     request_sent: bool,
//     move_buffer: Option<Move>,
// }

// impl Player for RemotePlayer {
//     fn request_move(&mut self, game: &Game) {
//         if !self.request_sent {
//             send_http(game.pos.clone()); // async network
//             self.request_sent = true;
//         }
//     }

//     fn poll_move(&mut self) -> Option<Move> {
//         check_response() // returns Option<Move>
//     }
// }
