use crate::engine::move_gen;
use crate::engine::position::*;
use crate::engine::types::{Move, MoveList};
use crate::engine::utils;

use super::player::*;

pub struct GameState {
    pos: Position,

    legal_moves: MoveList,

    players: [Box<dyn Player>; 2],

    // undo and redo
    undo_stack: Vec<(Move, Snapshot)>,
    redo_stack: Vec<(Move, Snapshot)>,
}

impl GameState {
    pub fn new() -> Self {
        let pos = Position::new();

        let mut game = Self {
            pos,
            legal_moves: MoveList::new(),
            players: [Box::new(NullPlayer), Box::new(NullPlayer)],
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        game.post_move();
        game
    }

    pub fn set_white(&mut self, player: Box<dyn Player>) {
        self.players[0] = player;
    }

    pub fn set_black(&mut self, player: Box<dyn Player>) {
        self.players[1] = player;
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }

    pub fn fen(&self) -> String {
        self.pos.fen()
    }

    pub fn active_player(&mut self) -> &mut dyn Player {
        let side_to_move = self.pos.side_to_move.as_usize();
        &mut *self.players[side_to_move]
    }

    pub fn execute(&mut self, m: &String) -> bool {
        let m = utils::parse_move(m.as_str());
        if m.is_none() {
            return false;
        }

        let (from, to, promtion) = m.unwrap();
        let legal_moves = move_gen::legal_moves(&self.pos);
        for m in legal_moves.iter() {
            if m.from_sq() == from && m.to_sq() == to && m.get_promotion() == promtion {
                let m = m.clone();
                let snapshot = self.pos.make_move(m);
                self.post_move();

                self.undo_stack.push((m, snapshot));
                self.redo_stack.clear();
                return true;
            }
        }
        return false;
    }

    pub fn game_over(&self) -> bool {
        self.legal_moves.count() == 0
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.redo_stack.len() > 0
    }

    pub fn undo(&mut self) -> bool {
        if let Some((m, snapshot)) = self.undo_stack.pop() {
            self.pos.unmake_move(m, &snapshot);
            self.post_move();

            self.redo_stack.push((m, snapshot));
            return true;
        }

        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some((m, snapshot)) = self.redo_stack.pop() {
            self.pos.make_move(m);
            self.post_move();

            self.undo_stack.push((m, snapshot));
            return true;
        }

        false
    }

    fn post_move(&mut self) {
        self.legal_moves = move_gen::legal_moves(&self.pos);
    }
}

// #[wasm_bindgen]
// pub struct WasmGame {
//     game: Game,
// }

// #[wasm_bindgen]
// impl WasmGame {
//     pub fn new() -> WasmGame {
//         WasmGame {
//             game: Game {
//                 pos: Position::start(),
//                 players: [Box::new(WebPlayer::default()), Box::new(AiPlayer)],
//                 turn: Color::White,
//                 history: vec![],
//             },
//         }
//     }

//     pub fn tick(&mut self) {
//         self.game.tick();
//     }

//     pub fn inject_move(&mut self, mv: Move) {
//         if let Some(web_player) = self.game.players[0].as_any().downcast_mut::<WebPlayer>() {
//             web_player.inject_move(mv);
//         }
//     }
// }

// let game = WasmGame.new();

// function gameLoop() {
//     game.tick();
//     requestAnimationFrame(gameLoop);
// }
// gameLoop();

// canvas.addEventListener('click', ev => {
//     let move = calc_move_from_click(ev);
//     game.inject_move(move);
// });
