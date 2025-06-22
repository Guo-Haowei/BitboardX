use crate::core::position::{Position, UndoState};
use crate::core::{move_gen, types::*};
use crate::engine::Engine;
use crate::engine::book::*;
use crate::engine::eval::Evaluation;

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

const DRAW_PENALTY: i32 = -50;
const IMMEDIATE_MATE_SCORE: i32 = 40000;

pub struct Searcher {
    evaluation_count: u64,
    // @TODO: cutoff move history
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
        Self { evaluation_count: 0 }
    }

    fn make_move(&mut self, pos: &mut Position, mv: &Move) -> UndoState {
        let mv = mv.clone();
        let undo_state = pos.make_move(mv);
        undo_state
    }

    fn unmake_move(&mut self, pos: &mut Position, mv: &Move, undo_state: &UndoState) {
        let mv = mv.clone();
        pos.unmake_move(mv, undo_state);
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
        max_ply: u8,
        ply_remaining: u8,
        mut alpha: i32,
        beta: i32,
    ) -> (i32, Move) {
        // @TODO: refactor draw detection and mate detection
        let key = engine.pos.zobrist();
        // threefold draw
        let repetition = engine.repetition_count(key);
        if repetition >= 2 {
            debug_assert!(repetition == 2); // if we make this move, it will be a draw
            log::debug!("Repetition detected: {}", engine.pos.fen());
            return (DRAW_PENALTY, Move::null());
        }

        // 50-move rule draw
        if engine.pos.halfmove_clock >= 100 {
            log::debug!("50-move rule draw detected: {}", engine.pos.fen());
            return (DRAW_PENALTY, Move::null());
        }

        let move_list = move_gen::legal_moves(&engine.pos);

        if move_list.len() == 0 {
            let score = if engine.pos.is_in_check() {
                // shallower checkmate should have higher score
                // because it's a position where the side to move is losing,
                // so we negate the score
                -(IMMEDIATE_MATE_SCORE + ply_remaining as i32)
            } else {
                DRAW_PENALTY
            };
            return (score, Move::null());
        }

        if ply_remaining == 0 {
            return (self.evaluate(&engine.pos), Move::null());
        }

        let move_list = sort_moves(&engine.pos, &move_list);

        let mut best_move = Move::null();
        let mut best_score = MIN;

        for mv in move_list.iter() {
            let undo_state = self.make_move(&mut engine.pos, mv);

            let (score, _) = self.negamax(engine, max_ply, ply_remaining - 1, -beta, -alpha);
            let score = -score; // Negate the score for the opponent

            self.unmake_move(&mut engine.pos, mv, &undo_state);

            if score > best_score {
                best_score = score;
                best_move = *mv;
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break; // beta cut-off
            }
        }

        (best_score, best_move)
    }

    pub fn find_best_move(&mut self, engine: &mut Engine, depth: u8) -> Option<Move> {
        debug_assert!(depth > 0);

        let move_list = move_gen::legal_moves(&engine.pos);
        if move_list.len() == 0 {
            return None; // no legal moves
        }

        // @TODO: add ply optimization, if there are more than 20 plys, it's unlikely to find a book move
        const USE_BOOK: bool = true;
        if USE_BOOK {
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
        }

        let (score, mv) = self.negamax(engine, depth, depth, MIN, MAX);

        if mv.is_null() {
            return None;
        }

        log::debug!(
            "evaluated {} node, best move found: {} (score: {})",
            self.evaluation_count,
            mv.to_string(),
            score
        );

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
