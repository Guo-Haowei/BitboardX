use crate::core::position::Position;
use crate::core::types::*;
use crate::engine::eval::get_piece_value;
use crate::engine::search::SearchContext;

// @TODO: inplace sorting
pub fn sort_moves(
    pos: &Position,
    ctx: &SearchContext,
    move_list: &MoveList,
    ply: u8,
    prev_best_move: Move,
    cached_mv: Move,
) -> Vec<Move> {
    fn move_score_guess(
        ctx: &SearchContext,
        pos: &Position,
        ply: u8,
        mv: Move,
        prev_best_move: Move,
        cached_mv: Move,
    ) -> i32 {
        debug_assert!(mv != Move::null(), "move cannot be null");

        // move is in transposition table, give it a high score
        if mv == cached_mv {
            return 30_000;
        }

        // move is the previous depth best move, give it a high score
        if mv == prev_best_move {
            return 20_000;
        }

        //     Priority:
        //   1. TT move             → score 30_000
        //   2. Previous best move  → score 20_000
        //   3. Killer move         → score 10_000
        //   4. Capture (MVV-LVA)   → 5_000 + value
        //   5. Quiet               → 0

        let move_type = mv.get_type();
        let src_sq = mv.src_sq();
        let dst_sq = mv.dst_sq();
        let color = pos.state.side_to_move;
        let opponent = color.flip();
        let src_piece = pos.get_piece_at(src_sq);
        let captured_piece = if move_type == MoveType::EnPassant {
            Piece::get_piece(opponent, PieceType::PAWN)
        } else {
            pos.get_piece_at(dst_sq)
        };

        if captured_piece == Piece::NONE {
            if ctx.is_killer(ply, mv) {
                return 10_000; // Killer move
            }
        }

        let src_piece_value = get_piece_value(src_piece.get_type());

        let mut score = 0;

        // prioritize capture high value piece with low value piece
        if captured_piece != Piece::NONE {
            let captured_piece_value = get_piece_value(captured_piece.get_type());
            score = 10 * captured_piece_value - src_piece_value;
        }

        // promote a pawn is also a good move
        if move_type == MoveType::Promotion {
            let promo_piece = mv.get_promotion().unwrap();
            score += get_piece_value(promo_piece);
        }

        // penalize moving a piece to a square that is attacked by an opponent piece
        if pos.state.attack_mask[opponent.as_usize()].test(dst_sq.as_u8()) {
            score = -src_piece_value;
        }

        score
    }

    // @TODO: inplace sorting without vector
    // @TODO: create move ordering class
    let mut scored_move: Vec<_> = move_list
        .iter()
        .map(|mv| (-move_score_guess(ctx, pos, ply, *mv, prev_best_move, cached_mv), mv.clone()))
        .collect();

    // Sort by score in descending order
    scored_move.sort_by_key(|(score, _)| *score);

    let sorted_moves: Vec<Move> = scored_move.into_iter().map(|(_, mv)| mv).collect();

    sorted_moves
}
