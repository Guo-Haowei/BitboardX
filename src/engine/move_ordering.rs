use crate::core::position::Position;
use crate::core::types::*;
use crate::engine::evaluation::get_piece_value;
use crate::engine::search::{PVLine, Searcher};

struct ScoredMove {
    mv: Move,
    score: i16,
}

pub fn sort_moves(
    pos: &Position,
    ctx: &Searcher,
    move_list: &mut MoveList,
    ply: u8,
    pv_line: &PVLine,
    cached_mv: Move,
) {
    fn move_score_guess(
        ctx: &Searcher,
        pos: &Position,
        ply: u8,
        mv: Move,
        pv_line: &PVLine,
        cached_mv: Move,
    ) -> i16 {
        debug_assert!(mv != Move::null(), "move cannot be null");

        // move is in transposition table, give it a high score
        if mv == cached_mv {
            return 30_000;
        }

        // move is the previous depth best move, give it a high score
        if mv == pv_line[ply as usize] {
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
        let color = pos.side_to_move;
        let opponent = color.flip();
        let src_piece = pos.get_piece_at(src_sq);
        let captured_piece = if move_type == MoveType::EnPassant {
            Piece::get_piece(opponent, PieceType::PAWN)
        } else {
            pos.get_piece_at(dst_sq)
        };

        // @TODO: killer move ranking
        if captured_piece == Piece::NONE {
            if ctx.is_killer(ply, mv) {
                return 15_000; // Killer move
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
            score += get_piece_value(promo_piece) + 5_000; // promotion bonus
        }

        if move_type == MoveType::Castling {
            // castling is a special move, give it a bonus
            score += 5_00;
        }

        // penalize moving a piece to a square that is attacked by an opponent piece
        if pos.state.attack_mask[opponent.as_usize()].test(dst_sq.as_u8()) {
            score -= src_piece_value / 2;
        }

        score
    }

    let mut scored: Vec<ScoredMove> = move_list.moves[..move_list.len()]
        .iter()
        .map(|&mv| ScoredMove {
            mv,
            score: move_score_guess(ctx, pos, ply, mv, pv_line, cached_mv),
        })
        .collect();

    scored.sort_unstable_by_key(|m| -m.score);

    for (i, scored_move) in scored.iter().enumerate() {
        move_list.moves[i] = scored_move.mv;
    }
}
