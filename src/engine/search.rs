use crate::core::{move_gen, position::Position, types::*};
use crate::engine::Engine;
use crate::engine::book::*;
use crate::engine::evaluation::Evaluation;
use crate::engine::move_ordering::sort_moves;
use crate::engine::ttable::NodeType;
use crate::utils;

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

const DRAW_PENALTY: i32 = -50;
const IMMEDIATE_MATE_SCORE: i32 = 40000;
const MAX_PLY: usize = 64; // max depth for search, should be enough for most positions
// @TODO: add ply optimization, if there are more than 20 plys, it's unlikely to find a book move
const USE_BOOK: bool = true;

macro_rules! if_debug_search {
    ($e:expr) => {
        // @TODO: change to false if you want to disable debug search logging
        if true {
            $e
        }
    };
}

pub type PVLine = [Move; MAX_PLY];

pub struct Searcher {
    killer_moves: [[Option<Move>; 2]; MAX_PLY],
    pv_table: [PVLine; MAX_PLY],
    pv_length: [usize; MAX_PLY],

    timer: utils::Timer,
    time_limit: f64, // in milliseconds
    cancel: bool,

    // for debugging purposes
    pruned_count: u64,
    total_moves: u64,
    leaf_count: u64,
}

impl Searcher {
    pub fn new(time_limit: f64) -> Self {
        Self {
            killer_moves: [[None; 2]; MAX_PLY],
            pv_table: [[Move::null(); MAX_PLY]; MAX_PLY],
            pv_length: [0; MAX_PLY],
            timer: utils::Timer::new(),
            time_limit,
            cancel: false,
            pruned_count: 0,
            total_moves: 0,
            leaf_count: 0,
        }
    }

    pub fn should_cancel(&mut self) -> bool {
        if self.cancel {
            return true;
        }
        if self.timer.elapsed_ms() >= self.time_limit {
            log::debug!("Time limit reached, cancelling search");
            self.cancel = true;
            return true;
        }
        false
    }

    // @TODO: review killer
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

    fn evaluate(&mut self, pos: &Position) -> i32 {
        self.leaf_count += 1;

        let mut eval = Evaluation::new();
        // @TODO: change to i16
        eval.evaluate_position(pos) as i32
    }

    fn quiescence(&mut self, engine: &mut Engine, mut alpha: i32, beta: i32, depth: i32) -> i32 {
        if self.should_cancel() {
            return 0;
        }

        let eval = self.evaluate(&engine.state.pos);
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

        let move_list = move_gen::pseudo_legal_capture_moves(&engine.state.pos);

        let mut has_legal_moves = false;
        let side_to_move = engine.state.pos.side_to_move;
        for mv in move_list.iter().copied() {
            let (undo_state, ok) = engine.state.pos.make_move(mv);
            if !ok {
                engine.state.pos.unmake_move(mv, &undo_state);
                continue; // illegal move
            }

            has_legal_moves = true;

            engine.state.push_zobrist();
            let score = -self.quiescence(engine, -beta, -alpha, depth - 1);

            if self.should_cancel() {
                return 0; // cancel the search
            }

            engine.state.pop_zobrist();

            engine.state.pos.unmake_move(mv, &undo_state);

            if score >= beta {
                // @TODO: stats
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        // @TODO: revisit this logic, might want to add ply to it
        if !has_legal_moves {
            if engine.state.pos.is_in_check(side_to_move) {
                return -(IMMEDIATE_MATE_SCORE + depth);
            } else {
                return DRAW_PENALTY;
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
        pv_line: &PVLine,
    ) -> (i32, Move) {
        if self.should_cancel() {
            return (0, Move::null());
        }

        let key = *engine.state.zobrist_stack.last().unwrap();
        let alpha_orig = alpha;

        // --- 1) Check for repetition and 50-move rule ---
        if max_ply > ply_remaining {
            if engine.state.is_three_fold() {
                log::debug!("repetition detected at depth: {}", ply_remaining);
                return (DRAW_PENALTY, Move::null());
            }

            if engine.state.is_fifty_draw() {
                log::debug!("50-move rule draw detected: {}", engine.state.pos.fen());
                return (DRAW_PENALTY, Move::null());
            }
        }

        // --- 2) Probe transposition table ---
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
            debug_assert!(
                !cached_move.is_null(),
                "Transposition table entry should have a best move"
            );
        }

        // --- 3) Check depth cutoff (leaf node) ---
        if ply_remaining == 0 {
            return (self.quiescence(engine, alpha, beta, 4), Move::null());
        }

        // @NOTE: we pseudo-legal moves here for speed, the illegal moves will be filtered out later
        let mut move_list = move_gen::pseudo_legal_moves(&engine.state.pos);

        // --- 4) Move ordering ---
        sort_moves(&engine.state.pos, &self, &mut move_list, ply_remaining, pv_line, cached_move);
        let mut best_move = Move::null();
        let mut best_score = MIN;

        let ply = (max_ply - ply_remaining) as usize;

        // --- 5) Main search loop ---
        let mut has_legal_moves = false;
        let mut mv_left = move_list.len();
        let side_to_move = engine.state.pos.side_to_move.clone();
        for mv in move_list.iter().copied() {
            let (undo_state, ok) = engine.state.pos.make_move(mv);
            if !ok {
                engine.state.pos.unmake_move(mv, &undo_state);
                continue;
            }

            has_legal_moves = true;
            let captured_piece = engine.state.pos.state.captured_piece;

            engine.state.push_zobrist();
            let (score, _) =
                self.negamax(engine, max_ply, ply_remaining - 1, -beta, -alpha, pv_line);

            if self.should_cancel() {
                return (0, Move::null()); // cancel the search
            }

            let score = -score; // Negate the score for the opponent

            engine.state.pop_zobrist();
            engine.state.pos.unmake_move(mv, &undo_state);

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

                // Update PV
                self.pv_table[ply][ply] = mv;
                self.pv_length[ply] = 1;
                for i in 0..self.pv_length[ply + 1] {
                    self.pv_table[ply][ply + 1 + i] = self.pv_table[ply + 1][ply + 1 + i];
                    self.pv_length[ply] += 1;
                }
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break; // beta cut-off
            }
            mv_left -= 1;
        }

        // --- 6) Check for terminal node (mate/stalemate) ---
        if !has_legal_moves {
            let score = if engine.state.pos.is_in_check(side_to_move) {
                // shallower checkmate should have higher score
                // because it's a position where the side to move is losing,
                // so we negate the score
                -(IMMEDIATE_MATE_SCORE + ply_remaining as i32)
            } else {
                DRAW_PENALTY
            };
            return (score, Move::null());
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

        debug_assert!(!best_move.is_null(), "Best move should be valid");
        engine.tt.store(key, ply_remaining, best_score, node_type, best_move);

        (best_score, best_move)
    }

    fn find_book_move(&mut self, engine: &mut Engine, move_list: &MoveList) -> Option<Move> {
        if let Some(book_mv) = DEFAULT_BOOK.get_move(*engine.state.zobrist_stack.last().unwrap()) {
            for mv in move_list.iter().copied() {
                if mv.src_sq() == book_mv.src_sq()
                    && mv.dst_sq() == book_mv.dst_sq()
                    && mv.get_promotion() == book_mv.get_promotion()
                {
                    log::debug!("Found book move: {:?}", book_mv.to_string());
                    return Some(mv);
                }
            }
            log::debug!("book move is: {:?}", book_mv.to_string());
            log::debug!("FEN: {:?}", engine.state.pos.fen());
            panic!("Should not happen, book move not found in legal moves");
        }

        None
    }

    pub fn find_best_move_depth(&mut self, engine: &mut Engine, max_depth: u8) -> Option<Move> {
        debug_assert!(max_depth > 0, "Depth should be greater than 0");

        // @TODO: fix it?
        let move_list = move_gen::legal_moves(&mut engine.state.pos);
        if move_list.is_empty() {
            return None;
        }

        if USE_BOOK {
            let book_move = self.find_book_move(engine, &move_list);
            if book_move.is_some() {
                return book_move;
            }
        }

        let mut best_move = Move::null();

        for depth in 1..=max_depth {
            self.total_moves = 0;
            self.pruned_count = 0;
            self.leaf_count = 0;

            let prev_pv = self.pv_table[0];
            let (_, mv) = self.negamax(engine, depth, depth, MIN, MAX, &prev_pv);

            best_move = mv;
        }

        return Some(best_move);
    }

    pub fn find_best_move(&mut self, engine: &mut Engine) -> Option<Move> {
        // @TODO: fix it?
        let move_list = move_gen::legal_moves(&mut engine.state.pos);
        if move_list.is_empty() {
            return None;
        }

        if USE_BOOK {
            let book_move = self.find_book_move(engine, &move_list);
            if book_move.is_some() {
                return book_move;
            }
        }

        let mut depth = 1;
        let mut best_move = Move::null();
        let mut best_score = MIN;

        // iterative deepening
        while depth < MAX_PLY as u8 {
            if self.should_cancel() {
                break;
            }

            self.total_moves = 0;
            self.pruned_count = 0;
            self.leaf_count = 0;

            let prev_pv = self.pv_table[0];
            let (score, mv) = self.negamax(engine, depth, depth, MIN, MAX, &prev_pv);

            if self.should_cancel() {
                break;
            }

            debug_assert!(!mv.is_null(), "Best move should be valid");

            best_move = mv;
            best_score = score;

            depth += 1;
        }

        if_debug_search!({
            let pv_moves = &self.pv_table[0][0..self.pv_length[0]];
            let mut moves = String::new();
            for mv in pv_moves.iter() {
                if mv.is_null() {
                    break;
                }
                moves.push(' ');
                moves.push_str(&mv.to_string());
            }

            log::debug!(
                "moves: {}(score: {}) found in {} ms, at depth: {}, {} leaves evaluated, {}/{} ({}%) pruned",
                moves,
                best_score,
                self.timer.elapsed_ms(),
                depth,
                self.leaf_count,
                self.pruned_count,
                self.total_moves,
                self.pruned_count as f32 / self.total_moves as f32 * 100.0
            );

            log::debug!(
                "tt table {}/{}, {}% full, collisions: {}",
                engine.tt.count(),
                engine.tt.capacity(),
                (engine.tt.count() as f32 / engine.tt.capacity() as f32 * 100.0).round(),
                engine.tt.collision_count
            );
        });

        Some(best_move)
    }
}
