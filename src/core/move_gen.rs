use super::position::Position;
use super::types::*;

mod generator;
mod validation;

pub use generator::PAWN_EN_PASSANT_MASKS;
pub use generator::{calc_attack_map_and_checker, pseudo_legal_capture_moves, pseudo_legal_moves};

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

pub fn is_pseudo_move_legal(pos: &Position, mv: Move) -> bool {
    validation::is_pseudo_move_legal(pos, mv)
}

pub fn generate_pin_map(pos: &Position, color: Color) -> BitBoard {
    generator::generate_pin_map(pos, color)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::core::position::Position;
//     use std::collections::HashSet;

//     fn assert_eq_unordered<T: Eq + std::hash::Hash + std::fmt::Debug>(a: &[T], b: &[T]) {
//         let set_a: HashSet<_> = a.iter().collect();
//         let set_b: HashSet<_> = b.iter().collect();
//         assert_eq!(set_a, set_b);
//     }

//     #[test]
//     fn test_capture_moves() {
//         let fen = "7K/1b2Q1nn/8/N7/8/1r1B4/8/6rk w - - 0 1";
//         let pos = Position::from_fen(fen).unwrap();
//         let moves = capture_moves(&pos);
//         let moves = moves.iter().map(|mv| mv.to_string()).collect::<Vec<_>>();

//         let expected: Vec<String> = vec![
//             "a5b3".to_string(),
//             "a5b7".to_string(),
//             "d3h7".to_string(),
//             "e7b7".to_string(),
//             "e7g7".to_string(),
//             "h8h7".to_string(),
//         ];

//         assert_eq_unordered(&moves, &expected);
//     }
// }
