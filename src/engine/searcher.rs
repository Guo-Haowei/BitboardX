use crate::core::position::{Position, UndoState};
use crate::core::{move_gen, types::*};
use crate::engine::Engine;
use crate::engine::book::*;
use crate::engine::eval::Evaluation;

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

const DRAW_PENALTY: i32 = -50;
const MATE_PENALTY: i32 = -40000;

pub struct Searcher {
    evaluation_count: u64,

    move_sequence: Vec<Move>,
}

fn sort_moves(pos: &Position, move_list: &MoveList) -> Vec<Move> {
    // @TODO: create move ordering class
    use crate::engine::eval;
    let mut scored_move: Vec<_> =
        move_list.iter().map(|mv| (-eval::move_score_guess(pos, *mv), mv.clone())).collect();

    // Sort by score in descending order
    scored_move.sort_by_key(|(score, _)| *score);

    let sorted_moves: Vec<Move> = scored_move.into_iter().map(|(_, mv)| mv).collect();

    sorted_moves
}

impl Searcher {
    pub fn new() -> Self {
        Self { evaluation_count: 0, move_sequence: Vec::new() }
    }

    fn make_move(&mut self, pos: &mut Position, mv: &Move) -> UndoState {
        let mv = mv.clone();
        let undo_state = pos.make_move(mv);
        self.move_sequence.push(mv);
        undo_state
    }

    fn unmake_move(&mut self, pos: &mut Position, mv: &Move, undo_state: &UndoState) {
        let mv = mv.clone();
        pos.unmake_move(mv, undo_state);
        self.move_sequence.pop();
    }

    fn evaluate(&mut self, pos: &Position) -> i32 {
        self.evaluation_count += 1;

        let mut eval = Evaluation::new();
        eval.evaluate_position(pos)
    }

    // @TODO: quiescence search
    /// Quiescence Search: only search captures when depth = 0
    #[allow(dead_code)]
    fn quiescence_search(
        &mut self,
        pos: &mut Position,
        mut alpha: i32,
        beta: i32,
        depth: u8,
    ) -> i32 {
        let eval = self.evaluate(pos);

        if eval >= beta || depth == 0 {
            return beta;
        }

        alpha = alpha.max(eval);

        let move_list = move_gen::capture_moves(pos);
        let move_list = sort_moves(pos, &move_list);

        for mv in move_list.iter() {
            let undo_state = self.make_move(pos, mv);

            let score = -self.quiescence_search(pos, -beta, -alpha, depth - 1);

            self.unmake_move(pos, mv, &undo_state);

            alpha = alpha.max(score);
            if alpha >= beta {
                break; // beta cut-off
            }
        }

        alpha
    }

    fn negamax(
        &mut self,
        engine: &mut Engine,
        ply_remaining: u8,
        ply_max: u8,
        mut alpha: i32,
        beta: i32,
    ) -> (Move, i32) {
        let key: crate::core::zobrist::ZobristHash = engine.pos.zobrist();
        let _alpha_orig = alpha;

        let repetition = engine.repetition_count(key);
        // threefold repetition check
        if repetition >= 2 {
            debug_assert!(repetition == 2);
            return (Move::null(), DRAW_PENALTY);
        }

        // 50-move rule check
        if engine.pos.halfmove_clock > 99 {
            return (Move::null(), DRAW_PENALTY);
        }

        // checkmate or stalemate
        let move_list = move_gen::legal_moves(&engine.pos);
        if move_list.len() == 0 {
            let score = if engine.pos.is_in_check() {
                MATE_PENALTY - ply_remaining as i32 // move that leads to checkmate in fewest plys is better
            } else {
                DRAW_PENALTY
            };
            return (Move::null(), score);
        }

        if ply_remaining == 0 {
            let score = self.evaluate(&engine.pos);
            return (Move::null(), score);
        }

        let mut best_move = Move::null();

        // @TODO: probe transposition table for a move

        let move_list = sort_moves(&engine.pos, &move_list);

        let mut i = move_list.len() - 1;
        for mv in move_list.iter() {
            let undo_state = self.make_move(&mut engine.pos, mv);

            let (_, score) = self.negamax(engine, ply_remaining - 1, ply_max, -beta, -alpha);
            let score = -score; // negamax

            if ply_max == ply_remaining {
                log::debug!(
                    "move '{}' has score {} (ply remaining: {})",
                    mv.to_string(),
                    score,
                    ply_remaining
                );
            }

            self.unmake_move(&mut engine.pos, mv, &undo_state);

            if alpha <= score {
                alpha = score;
                // @TODO: two posible moves???
                best_move = mv.clone();
            }

            // Move was *too* good, opponent will choose a different move earlier on to avoid this position.
            // (Beta-cutoff / Fail high)
            if alpha >= beta {
                if ply_max == ply_remaining {
                    log::debug!("beta cut-off without evaluating {} moves", i);
                }
                return (best_move, beta);
            }
            i -= 1;
        }

        // @TODO: store the best move in transposition table

        (best_move, alpha)
    }

    pub fn find_best_move(&mut self, engine: &mut Engine, depth: u8) -> Option<Move> {
        debug_assert!(depth > 0);

        // @TODO: add ply optimization, if there are more than 20 plys, it's unlikely to find a book move
        const USE_BOOK: bool = true;
        if USE_BOOK {
            if let Some(book_mv) = DEFAULT_BOOK.get_move(engine.last_hash) {
                let move_list = move_gen::legal_moves(&engine.pos);
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
        }

        let (mv, score) = self.negamax(engine, depth, depth, MIN, MAX);

        if mv.is_null() {
            log::debug!("no best move found at depth: {}", depth);
            return None;
        }

        Some(mv)
    }
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
