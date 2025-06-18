use crate::core::{position::Position, types::*};

const PAWN_SCORE: i32 = 100;
const KNIGHT_SCORE: i32 = 320;
const BISHOP_SCORE: i32 = 330;
const ROOK_SCORE: i32 = 500;
const QUEEN_SCORE: i32 = 900;
const KING_SCORE: i32 = 0; // skip king score for simplicity

const PIECE_SCORES: [i32; PieceType::COUNT as usize] =
    [PAWN_SCORE, KNIGHT_SCORE, BISHOP_SCORE, ROOK_SCORE, QUEEN_SCORE, KING_SCORE];

fn get_piece_value(piece_type: PieceType) -> i32 {
    assert!(piece_type != PieceType::NONE, "Piece must not be NONE");
    PIECE_SCORES[piece_type.as_u8() as usize]
}

fn count_material(pos: &Position, color: Color) -> i32 {
    let mut score = 0;
    for i in 0..PieceType::COUNT {
        let piece_type = PieceType(i);
        let piece = Piece::get_piece(color, piece_type);
        let count = pos.bitboards[piece.as_usize()].count() as i32;
        score += count * get_piece_value(piece_type);
    }

    score
}

pub fn evaluate(pos: &Position) -> i32 {
    debug_assert!(pos.side_to_move == Color::WHITE || pos.side_to_move == Color::BLACK);
    let score = count_material(pos, Color::WHITE) - count_material(pos, Color::BLACK);
    match pos.side_to_move {
        Color::WHITE => score,
        Color::BLACK => -score, // return score in favor of the side to move
        _ => panic!("Invalid side to move"),
    }
}

pub fn move_score_guess(pos: &Position, mv: Move) -> i32 {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::position::Position;

    #[test]
    fn test_evaluate_position() {
        let pos = Position::from_fen("8/8/8/8/k1RbP2K/8/8/8 w - - 0 1").unwrap();

        let score = evaluate(&pos);
        assert_eq!(score, ROOK_SCORE + PAWN_SCORE - BISHOP_SCORE);

        let pos = Position::from_fen("8/8/8/8/k1RbP2K/8/8/8 b - - 0 1").unwrap();

        let score = evaluate(&pos);
        assert_eq!(score, BISHOP_SCORE - ROOK_SCORE - PAWN_SCORE);
    }
}
