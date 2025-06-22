use crate::core::position::Position;
use crate::core::types::*;
use crate::engine::eval::get_piece_value;

// @TODO: refactor
pub fn move_score_guess(pos: &Position, mv: Move, tt_mv: Option<Move>) -> i32 {
    // move is in transposition table, give it a high score
    if let Some(tt_mv) = tt_mv {
        if mv == tt_mv {
            return 100000;
        }
    }

    let move_type = mv.get_type();
    let src_sq = mv.src_sq();
    let dst_sq = mv.dst_sq();
    let color = pos.side_to_move;
    let opponent = color.opponent();
    let src_piece = pos.get_piece_at(src_sq);
    let captured_piece = if move_type == MoveType::EnPassant {
        Piece::get_piece(opponent, PieceType::PAWN)
    } else {
        pos.get_piece_at(dst_sq)
    };
    let src_piece_value = get_piece_value(src_piece.get_type());

    let mut guess = 0;

    // prioritize capture high value piece with low value piece
    if captured_piece != Piece::NONE {
        let captured_piece_value = get_piece_value(captured_piece.get_type());
        guess = 10 * captured_piece_value - src_piece_value;
    }

    // promote a pawn is also a good move
    if move_type == MoveType::Promotion {
        let promo_piece = mv.get_promotion().unwrap();
        guess += get_piece_value(promo_piece);
    }

    // penalize moving a piece to a square that is attacked by an opponent piece
    if pos.attack_mask[opponent.as_usize()].test(dst_sq.as_u8()) {
        guess = -src_piece_value;
    }

    guess
}

// @TODO: inplace sorting
pub fn sort_moves(pos: &Position, move_list: &MoveList, tt_mv: Option<Move>) -> Vec<Move> {
    // @TODO: create move ordering class
    let mut scored_move: Vec<_> = move_list
        .iter()
        .map(|mv| (-move_score_guess(pos, mv.unwrap(), tt_mv), mv.clone()))
        .collect();

    // Sort by score in descending order
    scored_move.sort_by_key(|(score, _)| *score);

    let sorted_moves: Vec<Move> = scored_move.into_iter().map(|(_, mv)| mv.unwrap()).collect();

    sorted_moves
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::core::move_gen;

    #[test]
    fn test_sort_moves_with_guess() {
        let fen = "7k/2P5/1P6/8/8/8/8/K7 w - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        let move_list = move_gen::legal_moves(&pos);

        let sorted_moves = sort_moves(&pos, &move_list, None);
        let expected_best_move =
            Move::new(Square::C7, Square::C8, MoveType::Promotion, Some(PieceType::QUEEN));

        assert_eq!(expected_best_move, sorted_moves[0]);
    }
}
