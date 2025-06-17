use crate::core::{move_gen, position::Position, types::*};

const PAWN_SCORE: i32 = 100;
const KNIGHT_SCORE: i32 = 320;
const BISHOP_SCORE: i32 = 330;
const ROOK_SCORE: i32 = 500;
const QUEEN_SCORE: i32 = 900;
const KING_SCORE: i32 = 0; // skip king score for simplicity

const PIECE_SCORES: [i32; PieceType::COUNT as usize] =
    [PAWN_SCORE, KNIGHT_SCORE, BISHOP_SCORE, ROOK_SCORE, QUEEN_SCORE, KING_SCORE];

fn count_material(pos: &Position, color: Color) -> i32 {
    let mut score = 0;
    for i in 0..PieceType::COUNT {
        let piece_type = PieceType(i);
        let piece = Piece::get_piece(color, piece_type);
        let count = pos.bitboards[piece.as_usize()].count() as i32;
        score += count * PIECE_SCORES[piece_type.0 as usize];
    }

    score
}

pub fn evaluate(pos: &Position) -> i32 {
    debug_assert!(pos.side_to_move == Color::WHITE || pos.side_to_move == Color::BLACK);
    let score = count_material(pos, Color::WHITE) - count_material(pos, Color::BLACK);
    match pos.side_to_move {
        Color::WHITE => score,
        Color::BLACK => -score, // return score in favor of the side to move
        _ => unreachable!(),
    }
}

/// evaluate() always returns the score in favor of the side to move
/// if it's black's turn, the score will be score of black pieces minus score of white pieces.
/// assume we have this tree, root is the current position, it's black's turn to move,
/// black has 3 possible moves, assume white will respond with a move that maximize it's score
/// black can make a move 1, and the best move white respond will be 6 (makes the score -6) for black
/// so the score of first move is -6 for mover(6 for opponent), similarly, the score of 2nd move is -9
/// the score of last move is 2, (-2 for opponent), so we want to pick last move to minimize the opponent's
/// max score
///                      -9
///                   /   |   \
///                 /     |     \
///               6       9      -2
///             / | \   / | \   / | \
///           4 -6  0 -8  7 -9  4  2  6

fn negamax(pos: &mut Position, depth: u8) -> i32 {
    if depth == 0 {
        return evaluate(pos);
    }

    let move_list = move_gen::legal_moves(pos);
    if move_list.len() == 0 {
        if pos.is_in_check(pos.side_to_move) {
            return i32::MIN;
        }
        return 0; // draw
    }

    let mut best_score = i32::MIN;

    for mv in move_list.iter() {
        let undo_state = pos.make_move(*mv);
        let score = -negamax(pos, depth - 1);
        pos.unmake_move(*mv, &undo_state);
        best_score = best_score.max(score);
    }

    best_score
}

pub fn search(pos: &mut Position, depth: u8) -> Option<Move> {
    debug_assert!(depth > 0);
    let move_list = move_gen::legal_moves(pos);
    if move_list.len() == 0 {
        return None; // no legal moves
    }

    let mut best_move = None;
    let mut best_score = i32::MIN;

    for mv in move_list.iter() {
        let undo_state = pos.make_move(*mv);
        let score = -negamax(pos, depth - 1);
        pos.unmake_move(*mv, &undo_state);

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);
        }
    }

    best_move
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
