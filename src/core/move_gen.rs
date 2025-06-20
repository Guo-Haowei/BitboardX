use crate::core::position::CheckerList;

use super::position::Position;
use super::types::*;

mod generator;
mod validation;

/// Pseudo-legal move generation
pub fn pseudo_legal_moves(pos: &Position) -> MoveList {
    let mut move_list = MoveList::new();

    let color = pos.side_to_move;
    let (start, end) = if color == Color::WHITE {
        (Piece::W_START, Piece::W_END)
    } else {
        (Piece::B_START, Piece::B_END)
    };

    for i in start..=end {
        let piece = Piece::new(i);

        for sq in pos.bitboards[i as usize].iter() {
            generator::pseudo_legal_moves_src_sq(pos, sq, piece, &mut move_list);
        }
    }

    move_list
}

/// Legal move generation
pub fn legal_moves(pos: &Position) -> MoveList {
    let pseudo_moves = pseudo_legal_moves(pos);
    let mut moves = MoveList::new();
    for mv in pseudo_moves.iter() {
        if validation::is_pseudo_move_legal(pos, mv.clone()) {
            moves.add(mv.clone());
        }
    }

    moves
}

/// Capture move generation
pub fn capture_moves(pos: &Position) -> MoveList {
    let pseudo_moves = pseudo_legal_moves(pos);
    let mut moves = MoveList::new();
    let opponent = pos.side_to_move.opponent();
    for mv in pseudo_moves.iter() {
        if validation::is_pseudo_move_legal(pos, mv.clone()) {
            let dst_sq = mv.dst_sq();
            if pos.occupancies[opponent.as_usize()].test(dst_sq.as_u8()) {
                moves.add(mv.clone());
            }
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

pub fn calc_attack_map_impl(
    pos: &Position,
    piece: Piece,
    opponent_king: Square,
    checkers: &mut CheckerList,
) -> BitBoard {
    let mut attack_map = BitBoard::new();

    let color = piece.color();
    for sq in pos.bitboards[piece.as_usize()].iter() {
        let attack_mask = generator::attack_mask_src_sq(pos, sq, piece);

        if attack_mask.test(opponent_king.as_u8()) {
            debug_assert!(pos.occupancies[color.opponent().as_usize()].test(opponent_king.as_u8()));
            debug_assert!(pos.get_color_at(opponent_king) == piece.color().opponent());

            checkers.add(sq);
        }

        attack_map |= attack_mask;
    }

    attack_map
}
