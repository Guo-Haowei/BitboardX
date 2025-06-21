use super::super::position::Position;
use super::super::types::*;
use crate::utils;

/* #region */
/// # Legal Moves When the King is in Check
///
/// When the king is in check, only moves that resolve the check are legal.
/// These fall into three categories:
///
/// ## 1. Move the King
/// - The king may move to any adjacent square that is **not attacked**
/// - The king may **capture the checking piece** if that square is safe
/// - This is the **only legal move** in the case of a **double check**
///
/// ## 2. Capture the Checking Piece (If Only One Checker)
/// - Any piece (including the king) may capture the checking piece if:
///   - The piece is **not pinned**, or
///   - It is **pinned but capturing along the pin line**, and does not expose the king
/// - The capture must remove the check **without revealing a new one**
///
/// ## 3. Block the Check (If Only One Checker and It's a Sliding Piece)
/// - A non-king piece may interpose between the king and the checker if:
///   - The checker is a **rook, bishop, or queen**
///   - The blocking square is available and not pinned in a way that exposes the king
/// - Not possible if:
///   - The checker is a **knight** or **pawn**
///   - The check is **delivered from an adjacent square**
///
/// ## Special Cases
/// - **Double Check**:
///   - Only king moves are legal
///   - Captures and blocks are not sufficient, as two threats exist simultaneously
///
/// - **Pinned Piece**:
///   - Cannot move off the pin line (line between the king and an enemy sliding piece)
///   - May only capture the checker **if the move stays on the pin line**
///
/// ## Summary
/// | Condition                  | Legal Actions                      |
/// |---------------------------|-------------------------------------|
/// | Single check              | Move king, capture checker, block   |
/// | Double check              | Move king only                      |
/// | Pinned piece              | Capture on pin line only (if legal) |
/// | Checker is knight/pawn    | Cannot be blocked                   |
/// | Checker is sliding piece  | Can be blocked                      |

pub fn is_pseudo_move_legal(pos: &Position, mv: Move) -> bool {
    let mover = pos.get_piece_at(mv.src_sq());
    let mover_type = mover.get_type();
    let mover_color = pos.side_to_move;
    debug_assert!(mover_type != PieceType::NONE, "Mover must be a valid piece");
    debug_assert!(mover.color() == mover_color, "Mover color must match position side to move");
    let attacker_color = mover_color.opponent();

    // if there are two checkers, only moving the king solves the check
    let checker = &pos.checkers[mover_color.as_usize()];
    let checker_count = checker.count();

    let src_sq = mv.src_sq();
    let dst_sq = mv.dst_sq();
    // if move king, check if the destination square is safe
    if mover_type == PieceType::KING {
        for i in 0..=1 {
            let checker = checker.get(i);
            if let Some(checker_sq) = checker {
                if checker_sq != dst_sq {
                    let checker = pos.get_piece_at(checker_sq);
                    match checker.get_type() {
                        PieceType::BISHOP | PieceType::ROOK | PieceType::QUEEN => {
                            if src_sq.same_line(dst_sq, checker_sq) {
                                return false; // can't move the king along the line of the checker, unless it's a capture
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let dst_sq_under_attack = pos.attack_mask[attacker_color.as_usize()].test(dst_sq.as_u8());
        assert!(
            !dst_sq_under_attack,
            "this should be filtered when generating the mask, put an assert here for safety"
        );
        return !dst_sq_under_attack;
    }

    if checker_count == 2 {
        return false;
    }

    let is_pinned = pos.is_square_pinned(src_sq, mover_color);
    let king_sq = pos.get_king_square(mover_color);
    if is_pinned {
        // if there's a checker, the pinned piece can't be moved
        match checker_count {
            0 => return src_sq.same_line(dst_sq, king_sq), // No checkers, the move is legal.
            1 => return false, // if there's a checker, moving the pin won't help
            _ => panic!("There should be at most 1 checkers at this point"),
        }
    }

    match checker.get(0) {
        Some(checker_sq) => {
            if mv.get_type() == MoveType::EnPassant {
                let captured_sq = mv.get_en_passant_capture();
                if checker_sq == captured_sq {
                    return true;
                }
            }
            // if the move captures the checking piece, it is legal
            // otherwise if it blocks the check, it's still legal
            if dst_sq == checker_sq {
                true
            } else {
                dst_sq.same_line_inclusive(king_sq, checker_sq)
            }
        }
        None => {
            if mv.get_type() == MoveType::EnPassant {
                // En passant is a special case, it can only be legal if it captures the checking piece
                return is_pseudo_en_passant_legal(pos, mv, mover_color);
            }

            debug_assert!(
                checker.get(1).is_none(),
                "There should be at most 1 checker at this point"
            );
            return true;
        }
    }
}

/// # En Passant Discovered Check Edge Case
///
/// En passant is the **only move in chess** where:
/// - The **captured piece is not on the destination square**
/// - The move can potentially **remove two blockers** on the same rank (or file),
///   exposing the king to a **discovered check**
///
/// ## Scenario:
/// Imagine this position (Black to move):
///
/// ```text
/// 8  . . . . . . . .
/// 7  . . . . . . . .
/// 6  . . . . . . . .
/// 5  . . . . . . . .
/// 4  R . . . . P p k    ← Rank 4
/// 3  . . . . . . . .
/// 2  . . . . . P . .
/// 1  . . . . . . . .
///    a b c d e f g h
/// ```
/// - White just played `f2-f4`
/// - En passant is now legal (`g4xf3`)
/// - The black king is on `h4`, and white rook is on `a4`
///
/// If Black plays `g4xf3 e.p.`:
/// - The **g4 pawn moves to f3**
/// - The **f4 pawn is removed**
/// - Now both f4 and g4 are empty, so the rook on a4 checks the king on h4
///
/// ✅ This move is **illegal** — it exposes the king to a discovered check
///
/// ## Optimization:
/// Instead of simulating the board state:
/// - Perform a **raycast** in both directions from the en passant square:
///   - If one side hits the king, and the other hits a sliding attacker (rook/queen),
///     then the en passant move is **illegal**
///
/// This is a rare but critical edge case for legal move generation in chess engines.

fn is_pseudo_en_passant_legal(pos: &Position, mv: Move, mover_color: Color) -> bool {
    debug_assert!(mv.get_type() == MoveType::EnPassant, "Move must be an en passant move");

    let captured_sq = mv.get_en_passant_capture();
    let (src_file, _) = mv.src_sq().file_rank();
    let (captured_file, captured_rank) = captured_sq.file_rank();

    debug_assert!(
        pos.get_piece_at(captured_sq) == Piece::get_piece(mover_color.opponent(), PieceType::PAWN),
        "En passant capture must have an enemy pawn on the square to capture"
    );

    let (f_min, f_max) = utils::min_max(src_file.0, captured_file.0);

    let mut pieces = [Piece::NONE; 2];

    for file in (Rank::_1.0..f_min).rev() {
        let sq = Square::make(File(file), captured_rank);
        let piece = pos.get_piece_at(sq);
        if piece.get_type() != PieceType::NONE {
            pieces[0] = piece;
            break;
        }
    }
    for file in f_max + 1..=Rank::_8.0 {
        let sq = Square::make(File(file), captured_rank);
        let piece = pos.get_piece_at(sq);
        if piece.get_type() != PieceType::NONE {
            pieces[1] = piece;
            break;
        }
    }

    let my_king = Piece::get_piece(mover_color, PieceType::KING);
    if pieces[0] != my_king && pieces[1] != my_king {
        return true;
    }

    let their_piece = if pieces[0] == my_king { pieces[1] } else { pieces[0] };

    let their_rook = Piece::get_piece(mover_color.opponent(), PieceType::ROOK);
    let their_queen = Piece::get_piece(mover_color.opponent(), PieceType::QUEEN);
    if their_piece == their_rook || their_piece == their_queen {
        return false;
    }

    true
}

/* #endregion */
