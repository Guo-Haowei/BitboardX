use crate::core::{move_gen, position::Position, types::*};

mod eval;

const MIN: i32 = i32::MIN + 1; // to avoid overflow when negating
const MAX: i32 = i32::MAX;

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

struct Minimax {
    pub node_evaluated: u64,
}

impl Minimax {
    pub fn alpha_beta(
        &mut self,
        pos: &mut Position,
        depth: u8,
        mut alpha: i32,
        beta: i32,
    ) -> (i32, Option<Move>) {
        if depth == 0 {
            return (eval::evaluate(pos), None);
        }

        let move_list = move_gen::legal_moves(pos);
        if move_list.len() == 0 {
            if pos.is_in_check(pos.side_to_move) {
                return (MIN, None);
            }
            return (0, None); // draw
        }

        let move_list = sort_moves(pos, &move_list);

        let mut best = MIN;
        let mut final_move = None;
        for mv in move_list.iter() {
            let undo_state = pos.make_move(*mv);
            let (eval, _) = self.alpha_beta(pos, depth - 1, -beta, -alpha);
            let eval = -eval;
            pos.unmake_move(*mv, &undo_state);

            self.node_evaluated += 1;

            best = best.max(eval);

            if eval >= alpha {
                alpha = eval;
                final_move = Some(*mv);
            }

            if alpha >= beta {
                return (best, Some(*mv));
            }
        }

        (best, final_move)
    }
}

fn sort_moves(pos: &Position, move_list: &MoveList) -> Vec<Move> {
    let mut scored_move: Vec<_> =
        move_list.iter().map(|mv| (-eval::move_score_guess(pos, *mv), mv.clone())).collect();

    // Sort by score in descending order
    scored_move.sort_by_key(|(score, _)| *score);

    let sorted_moves: Vec<Move> = scored_move.into_iter().map(|(_, mv)| mv).collect();
    sorted_moves
}

pub fn search(pos: &mut Position, depth: u8) -> Option<Move> {
    debug_assert!(depth > 0);
    let move_list = move_gen::legal_moves(pos);
    if move_list.len() == 0 {
        return None; // no legal moves
    }

    let mut alpha_beta = Minimax { node_evaluated: 0 };

    let (_, mv2) = alpha_beta.alpha_beta(pos, depth, MIN, MAX);

    mv2
}

#[cfg(test)]

mod tests {
    use super::*;

    fn no_pruning(pos: &mut Position, depth: u8) -> (i32, Option<Move>) {
        if depth == 0 {
            return (eval::evaluate(pos), None);
        }

        let move_list = move_gen::legal_moves(pos);
        if move_list.len() == 0 {
            if pos.is_in_check(pos.side_to_move) {
                return (MIN, None);
            }
            return (0, None); // draw
        }

        let mut best_eval = MIN;
        let mut final_move = None;
        for mv in move_list.iter() {
            let undo_state = pos.make_move(*mv);
            let (eval, _) = no_pruning(pos, depth - 1);
            let eval = -eval;
            pos.unmake_move(*mv, &undo_state);

            if eval >= best_eval {
                best_eval = eval;
                final_move = Some(*mv);
            }
        }

        (best_eval, final_move)
    }

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

    #[test]
    fn test_alpha_beta_proning_correctness() {
        let fen = "8/8/8/8/k1RbP2K/8/8/8 w - - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        let depth = 3;

        let mut minimax = Minimax { node_evaluated: 0 };
        let (_, mv1) = minimax.alpha_beta(&mut pos, depth, MIN, MAX);
        let (_, mv2) = no_pruning(&mut pos, depth);

        let mv1 = mv1.unwrap();
        let mv2 = mv2.unwrap();
        assert_eq!(mv1, mv2, "Minimax raw and alpha-beta best moves do not match");
    }
}
