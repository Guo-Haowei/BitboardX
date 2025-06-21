use crate::core::{move_gen, position::Position, types::*};
use crate::engine::book::*;
use crate::engine::{Engine, eval};

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

const DRAW_PENALTY: i32 = 50;

// @TODO: quiescence search
/// Quiescence Search: only search captures when depth = 0
#[allow(dead_code)]
fn quiescence_search(pos: &mut Position, mut alpha: i32, beta: i32, depth: u8) -> i32 {
    let eval = eval::evaluate(pos);

    if eval >= beta || depth == 0 {
        return beta;
    }

    alpha = alpha.max(eval);

    let move_list = move_gen::capture_moves(pos);
    let move_list = sort_moves(pos, &move_list);

    for mv in move_list.iter() {
        let undo_state = pos.make_move(*mv);
        let score = -quiescence_search(pos, -beta, -alpha, depth - 1);
        pos.unmake_move(*mv, &undo_state);

        // self.node_evaluated += 1;

        alpha = alpha.max(score);
        if alpha >= beta {
            break; // beta cut-off
        }
    }

    alpha
}

fn alpha_beta_helper(engine: &mut Engine, mut alpha: i32, beta: i32, depth: u8) -> i32 {
    let key = engine.pos.zobrist();
    // threefold repetition check
    let repetition = engine.repetition_count(key);
    if repetition >= 2 {
        debug_assert!(repetition == 2); // if we make this move, it will be a draw
        log::debug!("Repetition detected: {}", engine.pos.fen());
        return -DRAW_PENALTY; // draw
    }

    let move_list = move_gen::legal_moves(&engine.pos);

    if move_list.len() == 0 {
        return if engine.pos.is_in_check() {
            MIN
        } else {
            -DRAW_PENALTY
            // stalemate
        };
    }

    if depth == 0 {
        // use a hard code depth of 4 for quiescence search
        return eval::evaluate(&engine.pos);
    }

    let move_list = sort_moves(&engine.pos, &move_list);

    for mv in move_list.iter() {
        let undo_state = engine.pos.make_move(*mv);
        let score = -alpha_beta_helper(engine, -beta, -alpha, depth - 1);
        engine.pos.unmake_move(*mv, &undo_state);

        // self.node_evaluated += 1;

        alpha = alpha.max(score);
        if alpha >= beta {
            break; // beta cut-off
        }
    }

    alpha
}

fn alpha_beta(engine: &mut Engine, depth: u8) -> Option<Move> {
    debug_assert!(depth > 0);
    let move_list = move_gen::legal_moves(&engine.pos);
    if move_list.len() == 0 {
        return None; // no legal moves
    }

    let mut alpha = MIN;
    let mut final_move = None;

    let move_list = sort_moves(&engine.pos, &move_list);

    for mv in move_list.iter() {
        let undo_state = engine.pos.make_move(*mv);
        let score = -alpha_beta_helper(engine, alpha, MAX, depth - 1);
        engine.pos.unmake_move(*mv, &undo_state);

        // self.node_evaluated += 1;

        if score >= alpha {
            alpha = score;
            final_move = Some(*mv);
        }
    }

    final_move
}

fn sort_moves(pos: &Position, move_list: &MoveList) -> Vec<Move> {
    let mut scored_move: Vec<_> =
        move_list.iter().map(|mv| (-eval::move_score_guess(pos, *mv), mv.clone())).collect();

    // Sort by score in descending order
    scored_move.sort_by_key(|(score, _)| *score);

    let sorted_moves: Vec<Move> = scored_move.into_iter().map(|(_, mv)| mv).collect();
    sorted_moves
}

pub fn find_best_move(engine: &mut Engine, depth: u8) -> Option<Move> {
    debug_assert!(depth > 0);

    let move_list = move_gen::legal_moves(&engine.pos);
    if move_list.len() == 0 {
        return None; // no legal moves
    }

    // @TODO: add ply optimization, if there are more than 20 plys, it's unlikely to find a book move
    if let Some(book_mv) = DEFAULT_BOOK.get_move(engine.last_hash) {
        for mv in move_list.iter() {
            if mv.src_sq() == book_mv.src_sq()
                && mv.dst_sq() == book_mv.dst_sq()
                && mv.get_promotion() == book_mv.get_promotion()
            {
                return Some(mv.clone());
            }
        }
        log::debug!("book move is: {:?}", book_mv.to_string());
        log::debug!("FEN: {:?}", engine.pos.fen());
        panic!("Should not happen, book move not found in legal moves");
    }

    log::trace!("Searching for best move at depth: {}", depth);

    let mv = alpha_beta(engine, depth);

    log::trace!("Best move found: {:?}", mv.to_string());

    mv
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_sort_moves_with_guess() {
        let fen = "7k/2P5/1P6/8/8/8/8/K7 w - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        let move_list = move_gen::legal_moves(&pos);

        let sorted_moves = sort_moves(&pos, &move_list);
        let expected_best_move =
            Move::new(Square::C7, Square::C8, MoveType::Promotion, Some(PieceType::QUEEN));

        assert_eq!(expected_best_move, sorted_moves[0]);
    }
}
