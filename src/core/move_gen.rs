use crate::core::move_gen::generator::pseudo_legal_capture_moves;

use super::position::Position;
use super::types::*;

mod generator;
mod validation;

pub use generator::PAWN_EN_PASSANT_MASKS;
pub use generator::calc_attack_map_and_checker;
pub use generator::pseudo_legal_moves;

/// Legal move generation
pub fn legal_moves(pos: &Position) -> MoveList {
    let pseudo_moves = pseudo_legal_moves(pos);
    let mut moves = MoveList::new();
    for mv in pseudo_moves.iter().copied() {
        if validation::is_pseudo_move_legal(pos, mv) {
            moves.add(mv);
        }
    }

    moves
}

/// Legal capture move generation
pub fn capture_moves(pos: &Position) -> MoveList {
    let pseudo_moves = pseudo_legal_capture_moves(pos);
    let mut moves = MoveList::new();
    for mv in pseudo_moves.iter().copied() {
        if validation::is_pseudo_move_legal(pos, mv) {
            moves.add(mv);
        }
    }

    moves
}

pub fn is_pseudo_move_legal(pos: &Position, mv: Move) -> bool {
    validation::is_pseudo_move_legal(pos, mv)
}

pub fn generate_pin_map(pos: &Position, color: Color) -> BitBoard {
    generator::generate_pin_map(pos, color)
}
