use crate::core::move_gen;
use crate::core::position::*;
use crate::core::types::{Move, MoveList};
use crate::core::utils;

use super::player::*;

pub struct GameState {
    pub pos: Position,

    pub legal_moves: MoveList,

    pub players: [Box<dyn Player>; 2],

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

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let pos = Position::from_fen(fen)?;
        let mut game = Self {
            pos,
            legal_moves: MoveList::new(),
            players: [Box::new(NullPlayer), Box::new(NullPlayer)],
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        game.post_move();
        Ok(game)
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

    pub fn execute(&mut self, mv: &String) -> Option<Move> {
        let mv = utils::parse_move(mv.as_str());
        if mv.is_none() {
            return None;
        }

        let (src, dst, promtion) = mv.unwrap();
        let legal_moves = move_gen::legal_moves(&self.pos);
        for mv in legal_moves.iter() {
            if mv.src_sq() == src && mv.dst_sq() == dst && mv.get_promotion() == promtion {
                let mv = mv.clone();
                let undo_state = self.pos.make_move(mv);
                self.post_move();

                self.undo_stack.push((mv, undo_state));
                self.redo_stack.clear();
                return Some(mv);
            }
        }

        None
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
        if let Some((mv, undo_state)) = self.undo_stack.pop() {
            self.pos.unmake_move(mv, &undo_state);
            self.post_move();

            self.redo_stack.push((mv, undo_state));
            return true;
        }

        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some((mv, undo_state)) = self.redo_stack.pop() {
            self.pos.make_move(mv);
            self.post_move();

            self.undo_stack.push((mv, undo_state));
            return true;
        }

        false
    }

    fn post_move(&mut self) {
        self.legal_moves = move_gen::legal_moves(&self.pos);
    }
}
