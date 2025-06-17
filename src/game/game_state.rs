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
    undo_stack: Vec<(Move, UndoState)>,
    redo_stack: Vec<(Move, UndoState)>,
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

        let (src, dst, promtion) = m.unwrap();
        let legal_moves = move_gen::legal_moves(&self.pos);
        for m in legal_moves.iter() {
            if m.src_sq() == src && m.dst_sq() == dst && m.get_promotion() == promtion {
                let m = m.clone();
                let undo_state = self.pos.make_move(m);
                self.post_move();

                self.undo_stack.push((m, undo_state));
                self.redo_stack.clear();
                return true;
            }
        }
        return false;
    }

    pub fn game_over(&self) -> bool {
        self.legal_moves.len() == 0
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.redo_stack.len() > 0
    }

    pub fn undo(&mut self) -> bool {
        if let Some((m, undo_state)) = self.undo_stack.pop() {
            self.pos.unmake_move(m, &undo_state);
            self.post_move();

            self.redo_stack.push((m, undo_state));
            return true;
        }

        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some((m, undo_state)) = self.redo_stack.pop() {
            self.pos.make_move(m);
            self.post_move();

            self.undo_stack.push((m, undo_state));
            return true;
        }

        false
    }

    fn post_move(&mut self) {
        self.legal_moves = move_gen::legal_moves(&self.pos);
    }
}
