use super::position::Position;
use super::types::*;

mod generator;
mod validation;

pub use generator::calc_attack_map_and_checker;

/// Pseudo-legal move generation
pub fn pseudo_legal_moves(pos: &Position) -> MoveList {
    let mut move_list = MoveList::new();

    let color = pos.state.side_to_move;
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
    for mv in pseudo_moves.iter().copied() {
        if validation::is_pseudo_move_legal(pos, mv) {
            moves.add(mv);
        }
    }

    moves
}

/// Capture move generation
pub fn capture_moves(pos: &Position) -> MoveList {
    let pseudo_moves = pseudo_legal_moves(pos);
    let mut moves = MoveList::new();
    let opponent = pos.state.side_to_move.flip();
    for mv in pseudo_moves.iter().copied() {
        let dst_sq = mv.dst_sq();
        if pos.state.occupancies[opponent.as_usize()].test(dst_sq.as_u8()) {
            if validation::is_pseudo_move_legal(pos, mv) {
                moves.add(mv);
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
