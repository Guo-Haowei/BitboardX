use crate::engine::move_gen::internal::pseudo_legal_move_from_to;

use super::board::*;
use super::position::Position;
use super::types::*;

mod internal;

/// Pseudo-legal move generation
pub fn pseudo_legal_moves(pos: &Position) -> Vec<Move> {
    let mut moves = Vec::new();

    for i in 0..64 {
        let sq = Square(i);
        let piece = pos.get_piece(sq);
        if piece.color() != pos.side_to_move {
            continue;
        }

        let bitboard = internal::pseudo_legal_move_from(pos, sq);
        if bitboard.none() {
            continue;
        }

        // @TODO: use trialing zeroes to optimize
        for j in 0..64 {
            if bitboard.test(j) {
                let to_sq = Square(j);
                let m = pseudo_legal_move_from_to(pos, sq, to_sq);
                moves.push(m);
            }
        }
    }

    moves
}

/// Legal move generation
pub fn legal_moves(pos: &mut Position) -> Vec<Move> {
    let mut moves = pseudo_legal_moves(pos);

    moves.retain(|m| internal::is_move_legal(pos, m));

    moves
}

pub fn is_move_legal(pos: &mut Position, m: &Move) -> bool {
    internal::is_move_legal(pos, m)
}

pub fn calc_attack_map_impl<const COLOR: u8, const START: u8, const END: u8>(
    pos: &Position,
) -> BitBoard {
    let mut attack_map = BitBoard::new();

    for i in START..=END {
        // pieces from W to B
        let bb = pos.bitboards[i as usize];
        for sq in 0..64 {
            if bb.test(sq) {
                attack_map |=
                    internal::pseudo_legal_attack_from(pos, Square(sq), Color::from(COLOR));
            }
        }
    }

    attack_map
}
