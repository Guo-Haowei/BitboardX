use crate::core::position::{Position, PositionState};
use crate::core::{move_gen, types::*};
use crate::engine::Engine;
use crate::engine::book::*;
use crate::engine::eval::Evaluation;
use crate::engine::move_ordering::sort_moves;
use crate::engine::ttable::NodeType;
use crate::utils;

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

const DRAW_PENALTY: i32 = -50;
const IMMEDIATE_MATE_SCORE: i32 = 40000;
const MAX_PLY: u8 = 64; // max depth for search, should be enough for most positions

macro_rules! if_debug_search {
    ($e:expr) => {
        // @TODO: change to false if you want to disable debug search logging
        if true {
            $e
        }
    };
}

pub struct SearchContext {
    prev_best_move: Move,

    killer_moves: [[Option<Move>; 2]; MAX_PLY as usize],

    // for debugging purposes
    pruned_count: u64,
    total_moves: u64,
    leaf_count: u64,
}

impl SearchContext {
    pub fn new() -> Self {
        Self {
            prev_best_move: Move::null(),
            killer_moves: [[None; 2]; MAX_PLY as usize],
            pruned_count: 0,
            total_moves: 0,
            leaf_count: 0,
        }
    }

    fn add_killer(&mut self, ply: u8, mv: Move) {
        let killers = &mut self.killer_moves[ply as usize];
        if killers[0] != Some(mv) {
            killers[1] = killers[0];
            killers[0] = Some(mv);
        }
    }

    pub fn is_killer(&self, ply: u8, mv: Move) -> bool {
        self.killer_moves[ply as usize].contains(&Some(mv))
    }

    fn make_move(&mut self, pos: &mut Position, mv: Move) -> PositionState {
        let undo_state = pos.make_move(mv);
        undo_state
    }

    fn unmake_move(&mut self, pos: &mut Position, mv: Move, undo_state: &PositionState) {
        pos.unmake_move(mv, undo_state);
    }

    fn evaluate(&mut self, pos: &Position) -> i32 {
        self.leaf_count += 1;

        let mut eval = Evaluation::new();
        eval.evaluate_position(pos)
    }

    fn quiescence(&mut self, engine: &mut Engine, mut alpha: i32, beta: i32, depth: i32) -> i32 {
        // @TODO: cancel

        // @TODO: order
        let move_list = move_gen::capture_moves(&engine.pos);

        let eval = self.evaluate(&engine.pos);
        if eval >= beta {
            // searchDiagnostics.numCutOffs++;
            return beta;
        }
        if eval > alpha {
            alpha = eval;
        }
        // because we can't cancel the search, add a depth parameter
        if depth == 0 {
            return eval;
        }

        if move_list.is_empty() {
            return self.evaluate(&engine.pos);
        }

        for mv in move_list.iter().copied() {
            let undo_state = self.make_move(&mut engine.pos, mv);
            let score = -self.quiescence(engine, -beta, -alpha, depth - 1);
            self.unmake_move(&mut engine.pos, mv, &undo_state);

            if score >= beta {
                // @TODO: stats
                return beta;
            }
            if score > alpha {
                alpha = score;
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
        mut beta: i32,
    ) -> (i32, Move) {
        // @TODO: refactor draw detection and mate detection
        let key = engine.pos.state.hash;
        let alpha_orig = alpha;
        let is_root = ply_remaining == max_ply;

        if max_ply > ply_remaining {
            // --- 1) Check for repetition and 50-move rule ---
            let repetition = engine.repetition_count(key);
            // threefold draw
            if repetition >= 3 {
                assert!(repetition == 3);
                log::debug!("repetition detected at depth: {}", ply_remaining);
                return (DRAW_PENALTY, Move::null());
            }

            // 50-move rule draw
            if engine.pos.state.halfmove_clock >= 100 {
                assert!(engine.pos.state.halfmove_clock == 100);
                log::debug!("50-move rule draw detected: {}", engine.pos.fen());
                return (DRAW_PENALTY, Move::null());
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
            return (score, Move::null());
        }

        // --- 3) Probe transposition table ---
        let mut cached_move = Move::null();
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
            cached_move = entry.best_move;
            assert!(!cached_move.is_null(), "Transposition table entry should have a best move");
        }

        // --- 4) Check depth cutoff (leaf node) ---
        if ply_remaining == 0 {
            return (self.quiescence(engine, alpha, beta, 4), Move::null());
        }

        // --- 5) Move ordering ---
        let prev_best_move = if is_root { self.prev_best_move } else { Move::null() };
        let move_list =
            sort_moves(&engine.pos, &self, &move_list, ply_remaining, prev_best_move, cached_move);
        let mut best_move = Move::null();
        let mut best_score = MIN;

        // --- 6) Main search loop ---
        let mut mv_left = move_list.len();
        for mv in move_list.iter().copied() {
            let undo_state = self.make_move(&mut engine.pos, mv);

            let captured_piece = engine.pos.state.captured_piece;

            let (score, _) = self.negamax(engine, max_ply, ply_remaining - 1, -beta, -alpha);
            let score = -score; // Negate the score for the opponent

            self.unmake_move(&mut engine.pos, mv, &undo_state);

            if score > best_score {
                if mv.get_type() == MoveType::Normal && captured_piece == Piece::NONE {
                    // this is a quiet move, so we can add it to the killer moves
                    self.add_killer(ply_remaining, mv);
                }

                // because we updated alpha every search,
                // from now on all moves will have at least alpha score
                // so we can only update best_move if score is strictly better than previous score
                best_score = score;
                best_move = mv;
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break; // beta cut-off
            }
            mv_left -= 1;
        }
        self.pruned_count += mv_left as u64;
        self.total_moves += move_list.len() as u64;

        // --- 7) Store result in transposition table ---
        let node_type = if best_score <= alpha_orig {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        assert!(!best_move.is_null(), "Best move should be valid");
        engine.tt.store(key, ply_remaining, best_score, node_type, best_move);

        (best_score, best_move)
    }

    pub fn find_best_move(&mut self, engine: &mut Engine, max_depth: u8) -> Option<Move> {
        debug_assert!(max_depth > 0);

        let move_list = move_gen::legal_moves(&engine.pos);
        if move_list.len() == 0 {
            return None; // no legal moves
        }

        // @TODO: add ply optimization, if there are more than 20 plys, it's unlikely to find a book move
        const USE_BOOK: bool = true;
        if USE_BOOK {
            if let Some(book_mv) = DEFAULT_BOOK.get_move(engine.pos.state.hash) {
                for mv in move_list.iter().copied() {
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

        // iterative deepening
        for depth in 1..MAX_PLY {
            // @TODO: time control
            if depth > max_depth {
                break;
            }

            self.total_moves = 0;
            self.pruned_count = 0;
            self.leaf_count = 0;

            let begin_time = utils::get_time();

            let (score, mv) = self.negamax(engine, depth, depth, MIN, MAX);
            assert!(!mv.is_null(), "Best move should be valid");

            self.prev_best_move = mv;
            if_debug_search!({
                let end_time = utils::get_time();
                log::debug!(
                    "move '{}'(score: {}) found in {} ms, at depth: {}, {} leaves evaluated, {}/{} ({}%) pruned",
                    mv.to_string(),
                    score,
                    (end_time - begin_time),
                    depth,
                    self.leaf_count,
                    self.pruned_count,
                    self.total_moves,
                    self.pruned_count as f32 / self.total_moves as f32 * 100.0
                );
            });
        }

        log::debug!(
            "tt table {}/{}, {}% full, collisions: {}",
            engine.tt.count(),
            engine.tt.capacity(),
            (engine.tt.count() as f32 / engine.tt.capacity() as f32 * 100.0).round(),
            engine.tt.collision_count
        );

        Some(self.prev_best_move)
    }
}
