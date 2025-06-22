use crate::core::position::{Position, UndoState};
use crate::core::{move_gen, types::*};
use crate::engine::Engine;
use crate::engine::book::*;
use crate::engine::eval::Evaluation;
use crate::engine::move_ordering::sort_moves;
use crate::engine::ttable::NodeType;

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

const DRAW_PENALTY: i32 = -50;
const IMMEDIATE_MATE_SCORE: i32 = 40000;

const DEBUG_PRINT: bool = false;
macro_rules! if_debug_print {
    ($e:expr) => {
        if DEBUG_PRINT {
            $e
        }
    };
}

pub struct Searcher {
    evaluation_count: u64,
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

    fn negamax(
        &mut self,
        engine: &mut Engine,
        max_ply: u8,
        ply_remaining: u8,
        mut alpha: i32,
        mut beta: i32,
    ) -> (i32, Option<Move>) {
        // @TODO: refactor draw detection and mate detection
        let key = engine.pos.zobrist();
        let alpha_orig = alpha;

        if max_ply > ply_remaining {
            // --- 1) Check for repetition and 50-move rule ---
            let repetition = engine.repetition_count(key);
            // threefold draw
            if repetition >= 3 {
                assert!(repetition == 3);
                log::debug!("repetition detected at depth: {}", ply_remaining);
                return (DRAW_PENALTY, None);
            }

            // 50-move rule draw
            if engine.pos.halfmove_clock >= 100 {
                assert!(engine.pos.halfmove_clock == 100);
                log::debug!("50-move rule draw detected: {}", engine.pos.fen());
                return (DRAW_PENALTY, None);
            }
        }

        // --- 2) Check for terminal node (mate/stalemate) ---
        let move_list = move_gen::legal_moves(&engine.pos);
        if move_list.is_empty() {
            let score = if engine.pos.is_in_check() {
                // shallower checkmate should have higher score
                // because it's a position where the side to move is losing,
                // so we negate the score
                -(IMMEDIATE_MATE_SCORE + ply_remaining as i32)
            } else {
                DRAW_PENALTY
            };
            return (score, None);
        }

        // --- 3) Probe transposition table ---
        let mut tt_move = None;
        if let Some(entry) = engine.tt.probe(key) {
            if entry.depth >= ply_remaining {
                let mut found = false;
                match entry.node_type {
                    NodeType::Exact => found = true,
                    NodeType::LowerBound => alpha = alpha.max(entry.score as i32),
                    NodeType::UpperBound => beta = beta.min(entry.score as i32),
                }
                if found || alpha >= beta {
                    return (entry.score, entry.best_move);
                }
            }
            tt_move = entry.best_move;
            assert!(tt_move.is_some(), "Transposition table entry should have a best move");
        }

        // --- 4) Check depth cutoff (leaf node) ---
        if ply_remaining == 0 {
            return (self.evaluate(&engine.pos), None);
        }

        // --- 5) Move ordering ---
        let move_list = sort_moves(&engine.pos, &move_list, tt_move);
        let mut best_move: Option<Move> = None;
        let mut best_score = MIN;

        // --- 6) Main search loop ---
        let mut mv_left = move_list.len();
        for mv in move_list.iter() {
            let undo_state = self.make_move(&mut engine.pos, mv);

            let (score, _) = self.negamax(engine, max_ply, ply_remaining - 1, -beta, -alpha);
            let score = -score; // Negate the score for the opponent

            self.unmake_move(&mut engine.pos, mv, &undo_state);

            if score >= best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                if_debug_print!(if max_ply - ply_remaining <= 2 && ply_remaining > 1 {
                    log::debug!("{}/{} nodes pruned", mv_left, move_list.len());
                });

                break; // beta cut-off
            }
            mv_left -= 1;
        }

        // --- 7) Store result in transposition table ---
        let node_type = if best_score <= alpha_orig {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        assert!(best_move.is_some(), "Best move should be valid");
        engine.tt.store(key, ply_remaining, best_score, node_type, best_move);

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
                    let mv = mv.unwrap();
                    if mv.src_sq() == book_mv.src_sq()
                        && mv.dst_sq() == book_mv.dst_sq()
                        && mv.get_promotion() == book_mv.get_promotion()
                    {
                        return Some(mv);
                    }
                }
                log::debug!("book move is: {:?}", book_mv.to_string());
                log::debug!("FEN: {:?}", engine.pos.fen());
                panic!("Should not happen, book move not found in legal moves");
            }
        }

        let (score, mv) = self.negamax(engine, depth, depth, MIN, MAX);

        assert!(mv.is_some(), "Best move should be valid");

        let mv = mv.unwrap();

        log::debug!(
            "evaluated {} node, best move found: {} (score: {})",
            self.evaluation_count,
            mv.to_string(),
            score
        );

        log::debug!(
            "tt table {}/{}, {}% full, collisions: {}",
            engine.tt.count(),
            engine.tt.capacity(),
            (engine.tt.count() as f32 / engine.tt.capacity() as f32 * 100.0).round(),
            engine.tt.collision_count
        );

        Some(mv)
    }
}
