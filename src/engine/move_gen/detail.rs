use super::super::board::*;
use super::super::position::{Position, SmallSquareList};
use super::super::types::{Color, Piece, PieceType};
use super::super::utils;

pub fn pseudo_legal_moves_from_sq(
    move_list: &mut MoveList,
    piece: Piece,
    pos: &Position,
    sq: Square,
) {
    let color = piece.color();

    let my = pos.occupancies[color.as_usize()];
    let enemy = pos.occupancies[color.opponent().as_usize()];

    match piece {
        Piece::W_PAWN => pseudo_legal_move_pawn::<{ Color::WHITE.as_u8() }>(move_list, sq, pos),
        Piece::B_PAWN => pseudo_legal_move_pawn::<{ Color::BLACK.as_u8() }>(move_list, sq, pos),
        Piece::W_KNIGHT | Piece::B_KNIGHT => pseudo_legal_move_knight(move_list, sq, my),
        Piece::W_ROOK | Piece::B_ROOK => pseudo_legal_move_rook(move_list, sq, my, enemy),
        Piece::W_BISHOP | Piece::B_BISHOP => pseudo_legal_move_bishop(move_list, sq, my, enemy),
        Piece::W_QUEEN | Piece::B_QUEEN => pseudo_legal_move_queen(move_list, sq, my, enemy),
        Piece::W_KING => pseudo_legal_move_king::<{ Color::WHITE.as_u8() }>(move_list, sq, pos),
        Piece::B_KING => pseudo_legal_move_king::<{ Color::BLACK.as_u8() }>(move_list, sq, pos),
        _ => panic!("Invalid piece type: {:?}", piece),
    }
}

// @TODO: deprecate this
/// Pseudo-legal move generation for a square
pub fn pseudo_legal_from_sq_impl<const ATTACK_ONLY: bool>(
    pos: &Position,
    sq: Square,
    color: Color,
) -> BitBoard {
    let piece = pos.get_piece_at(sq);

    let my_occupancy = pos.occupancies[color.as_usize()];
    let enemy_occupancy = pos.occupancies[color.opponent().as_usize()];

    match piece {
        Piece::W_PAWN => move_mask_pawn::<{ Color::WHITE.as_u8() }, ATTACK_ONLY>(sq, pos),
        Piece::B_PAWN => move_mask_pawn::<{ Color::BLACK.as_u8() }, ATTACK_ONLY>(sq, pos),
        Piece::W_ROOK | Piece::B_ROOK => move_mask_rook(sq, my_occupancy, enemy_occupancy),
        Piece::W_BISHOP | Piece::B_BISHOP => move_mask_bishop(sq, my_occupancy, enemy_occupancy),
        Piece::W_QUEEN | Piece::B_QUEEN => move_mask_queen(sq, my_occupancy, enemy_occupancy),
        Piece::W_KNIGHT | Piece::B_KNIGHT => move_mask_knight(sq, my_occupancy),
        Piece::W_KING => move_mask_king::<{ Color::WHITE.as_u8() }, ATTACK_ONLY>(sq, pos),
        Piece::B_KING => move_mask_king::<{ Color::BLACK.as_u8() }, ATTACK_ONLY>(sq, pos),
        Piece::NONE => BitBoard::new(),
        _ => {
            panic!("Invalid piece type: {:?}", piece);
        }
    }
}

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

pub fn is_pseudo_move_legal(pos: &mut Position, m: &Move) -> bool {
    let mover = pos.get_piece_at(m.from_sq());
    let mover_type = mover.get_type();
    let mover_color = pos.side_to_move;
    debug_assert!(mover_type != PieceType::None, "Mover must be a valid piece");
    debug_assert!(mover.color() == mover_color, "Mover color must match position side to move");
    let attacker_color = mover_color.opponent();

    let to_sq = m.to_sq();
    // if move king, check if the destination square is safe
    if mover_type == PieceType::King {
        let to_sq_under_attack =
            pos.attack_map_color[attacker_color.as_usize()].test(to_sq.as_u8());
        assert!(
            !to_sq_under_attack,
            "this should be filtered when generating the mask, put an assert here for safety"
        );
        return !to_sq_under_attack;
    }

    // if there are two checkers, only moving the king solves the check
    let checker = &pos.checkers[mover_color.as_usize()];
    let checker_count = checker.count();

    if checker_count == 2 {
        return false;
    }

    let from_sq = m.from_sq();
    let is_pinned = pos.is_square_pinned(from_sq, mover_color);
    let king_sq = pos.get_king_square(mover_color);
    if is_pinned {
        // if there's a checker, the pinned piece can't be moved
        match checker_count {
            0 => return from_sq.same_line(to_sq, king_sq), // No checkers, the move is legal.
            1 => return false, // if there's a checker, moving the pin won't help
            _ => panic!("There should be at most 1 checkers at this point"),
        }
    }

    match checker.get(0) {
        Some(checker_sq) => {
            if m.get_type() == MoveType::EnPassant {
                let captured_sq = m.get_en_passant_capture();
                if checker_sq == captured_sq {
                    return true;
                }
            }
            // if the move captures the checking piece, it is legal
            // otherwise if it blocks the check, it's still legal
            if to_sq == checker_sq { true } else { to_sq.same_line_inclusive(king_sq, checker_sq) }
        }
        None => {
            if m.get_type() == MoveType::EnPassant {
                // En passant is a special case, it can only be legal if it captures the checking piece
                return is_pseudo_en_passant_legal(pos, m, mover_color);
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

fn is_pseudo_en_passant_legal(pos: &Position, m: &Move, mover_color: Color) -> bool {
    debug_assert!(m.get_type() == MoveType::EnPassant, "Move must be an en passant move");

    let captured_sq = m.get_en_passant_capture();
    let (from_file, _) = m.from_sq().file_rank();
    let (captured_file, captured_rank) = captured_sq.file_rank();

    debug_assert!(
        pos.get_piece_at(captured_sq) == Piece::get_piece(mover_color.opponent(), PieceType::Pawn),
        "En passant capture must have an enemy pawn on the square to capture"
    );

    let (f_min, f_max) = utils::min_max(from_file as u8, captured_file as u8);

    let mut pieces = [Piece::NONE; 2];

    for file in (RANK_1..f_min).rev() {
        let sq = Square::make(file, captured_rank);
        let piece = pos.get_piece_at(sq);
        if piece.get_type() != PieceType::None {
            pieces[0] = piece;
            break;
        }
    }
    for file in f_max + 1..=RANK_8 {
        let sq = Square::make(file, captured_rank);
        let piece = pos.get_piece_at(sq);
        if piece.get_type() != PieceType::None {
            pieces[1] = piece;
            break;
        }
    }

    let my_king = Piece::get_piece(mover_color, PieceType::King);
    if pieces[0] != my_king && pieces[1] != my_king {
        return true;
    }

    let their_piece = if pieces[0] == my_king { pieces[1] } else { pieces[0] };

    let their_rook = Piece::get_piece(mover_color.opponent(), PieceType::Rook);
    let their_queen = Piece::get_piece(mover_color.opponent(), PieceType::Queen);
    if their_piece == their_rook || their_piece == their_queen {
        return false;
    }

    true
}

/* #endregion */

/* #region */
/// Computes the legal pawn moves from a given bitboard position,
/// including single and double pushes, captures, promotions, and en passant.
///
/// Pawn movement is asymmetric and depends on color. Each pawn moves forward
/// (toward the opponent’s side), captures diagonally, and promotes on the final rank.
///
/// ## Movement Types
///
/// - **Single Push**: 1-square forward (N for white, S for black)
/// - **Double Push**: 2-squares forward from the starting rank
/// - **Captures**:
///   - White: NE, NW
///   - Black: SE, SW
/// - **Promotion**: On reaching rank 8 (white) or rank 1 (black)
/// - **En Passant**: Special capture on adjacent file if opponent just advanced a pawn 2 squares
///
/// ## Directions
///
/// For white pawns:
/// - N  → Forward (single push)
/// - 2N → Double push from rank 2
/// - NE / NW → Diagonal captures
///
/// For black pawns:
/// - S  → Forward (single push)
/// - 2S → Double push from rank 7
/// - SE / SW → Diagonal captures

fn move_mask_pawn<const COLOR: u8, const ATTACK_ONLY: bool>(
    sq: Square,
    pos: &Position,
) -> BitBoard {
    let (_file, rank) = sq.file_rank();
    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();

    let opponent = COLOR ^ 1;

    let is_white = COLOR == Color::WHITE.as_u8();
    let is_black = COLOR != Color::WHITE.as_u8();

    if !ATTACK_ONLY {
        // Handle forward moves
        let next_bb = if is_white { shift_north(bb) } else { shift_south(bb) };

        if (next_bb & pos.occupancies[Color::BOTH.as_usize()]).none() {
            moves |= next_bb;
        }

        if (is_white && rank == RANK_2 || is_black && rank == RANK_7) && moves.any() {
            let next_bb = if is_white { shift_north(next_bb) } else { shift_south(next_bb) };
            if (next_bb & pos.occupancies[Color::BOTH.as_usize()]).none() {
                moves |= next_bb;
            }
        }

        // Handle en passant
        moves |= move_mask_pawn_ep::<COLOR>(pos, sq);
    }

    // Handle attacks moves
    let attack_left = if is_white { shift_nw(bb) } else { shift_sw(bb) };
    if ATTACK_ONLY || (attack_left & pos.occupancies[opponent as usize]).any() {
        moves |= attack_left;
    }
    let attack_right = if is_white { shift_ne(bb) } else { shift_se(bb) };
    if ATTACK_ONLY || (attack_right & pos.occupancies[opponent as usize]).any() {
        moves |= attack_right;
    }

    moves
}

fn move_mask_pawn_ep<const COLOR: u8>(pos: &Position, sq: Square) -> BitBoard {
    // Handle en passant
    let (file, rank) = sq.file_rank();
    let is_white = COLOR == Color::WHITE.as_u8();
    let is_black = COLOR != Color::WHITE.as_u8();

    if let Some(ep_sq) = pos.en_passant {
        debug_assert!(pos.get_piece_at(ep_sq) == Piece::NONE, "En passant square must be empty");
        let (ep_file, ep_rank) = ep_sq.file_rank();
        if (file as i32 - ep_file as i32).abs() == 1 {
            if is_white && rank == RANK_5 && ep_rank == RANK_6 {
                debug_assert!(pos.get_piece_at(Square(ep_sq.0 - 8)) == Piece::B_PAWN);
                return ep_sq.to_bitboard();
            }
            if is_black && rank == RANK_4 && ep_rank == RANK_3 {
                debug_assert!(pos.get_piece_at(Square(ep_sq.0 + 8)) == Piece::W_PAWN);
                return ep_sq.to_bitboard();
            }
        }
    }

    return BitBoard::new();
}

fn pseudo_legal_move_pawn<const COLOR: u8>(move_list: &mut MoveList, sq: Square, pos: &Position) {
    let mask = move_mask_pawn::<{ COLOR }, false>(sq, pos);
    let mut bb = mask;

    while bb.any() {
        let to_sq = bb.first_nonzero_sq();

        if check_if_promotion::<COLOR>(to_sq) {
            // Promotion move
            let promotion_types =
                [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight];
            for &promotion in &promotion_types {
                move_list.add(Move::new(sq, to_sq, MoveType::Promotion, Some(promotion)));
            }
        } else {
            let is_ep_capture =
                check_if_eq_capture::<COLOR>(pos, sq, to_sq, pos.get_piece_at(to_sq));
            let move_type = if is_ep_capture { MoveType::EnPassant } else { MoveType::Normal };
            move_list.add(Move::new(sq, to_sq, move_type, None));
        }
        bb.remove_first_nonzero_sq();
    }
}

fn check_if_promotion<const COLOR: u8>(to_sq: Square) -> bool {
    let (_, rank) = to_sq.file_rank();

    match rank {
        RANK_8 if COLOR == Color::WHITE.as_u8() => true,
        RANK_1 if COLOR == Color::BLACK.as_u8() => true,
        _ => false,
    }
}

fn check_if_eq_capture<const COLOR: u8>(
    pos: &Position,
    from_sq: Square,
    to_sq: Square,
    to: Piece,
) -> bool {
    if to.get_type() != PieceType::None {
        return false;
    }

    // 8 . . . . . k . . black pawn c7c5, c6 is empty, c5 has black pawn
    // 7 . . . . . . . .
    // 6 . . . . . . . .
    // 5 . . p P . . . .
    // 4 . . . . . . . .
    // 3 . . . . . . . .
    // 2 . . . . . . . .
    // 1 . . . . K . . .
    //   a b c d e f g h

    // if the to square is empty, but it still moves diagonally, then
    let (from_file, from_rank) = from_sq.file_rank();
    let (to_file, to_rank) = to_sq.file_rank();
    if from_file == to_file {
        return false;
    }
    debug_assert!((from_file as i8 - to_file as i8).abs() == 1);
    debug_assert!((from_rank as i8 - to_rank as i8).abs() == 1);

    if cfg!(debug_assertions) {
        let color = Color::from(COLOR);
        let enemy = if color == Color::WHITE { Piece::B_PAWN } else { Piece::W_PAWN };
        let enemy_sq = Square::make(to_file, from_rank);

        debug_assert!(
            pos.bitboards[enemy.as_usize()].test(enemy_sq.as_u8()),
            "En passant capture must have an enemy pawn on the square to capture"
        );
    }

    true
}

/* #endregion */

/* #region */
/// Computes the legal moves for sliding pieces (rook, bishop, and queen) from a given square on a bitboard.
///
/// Sliding pieces can move in straight lines across the board until they hit another piece or the board's edge.
/// The move set depends on the piece:
/// - **Rook**: moves along ranks and files (N, S, E, W)
/// - **Bishop**: moves along diagonals (NE, NW, SE, SW)
/// - **Queen**: combines rook and bishop movement
///
/// ## Movement Directions
///
/// - Rook:
///   - N  → North (up)
///   - S  → South (down)
///   - E  → East  (right)
///   - W  → West  (left)
///
/// - Bishop:
///   - NE → North-East
///   - NW → North-West
///   - SE → South-East
///   - SW → South-West
///
/// - Queen:
///   - All 8 directions: N, S, E, W, NE, NW, SE, SW

/// Pseudo-legal move generation for a sliding piece (rook, bishop, queen)
fn move_mask_sliding<const START: u8, const END: u8>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
    // color: Color,
) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();

    for i in START..END {
        let mut next_bb = SHIFT_FUNCS[i as usize](bb);

        while next_bb.any() {
            if (next_bb & my_occupancy).any() {
                break;
            }

            if (next_bb & enemy_occupancy).any() {
                moves |= next_bb;
                break;
            }

            moves |= next_bb;

            next_bb = SHIFT_FUNCS[i as usize](next_bb);
        }
    }

    moves
}

pub fn generate_pin_map(pos: &Position, color: Color) -> BitBoard {
    let mut pin_map = BitBoard::new();

    let occupied = pos.occupancies[Color::BOTH.as_usize()];
    let king_bb = pos.bitboards[Piece::get_piece(color, PieceType::King).as_usize()];

    for i in 0..8 {
        let mut next_bb = SHIFT_FUNCS[i as usize](king_bb);

        let mut squares = SmallSquareList::new();

        while next_bb.any() {
            if (next_bb & occupied).any() {
                squares.add(next_bb.first_nonzero_sq());
                if squares.count() == 2 {
                    break; // Found two pieces in this direction
                }
            }

            next_bb = SHIFT_FUNCS[i as usize](next_bb);
        }

        if squares.count() != 2 {
            continue; // No pin found in this direction
        }

        let sq0 = squares.get(0).unwrap();
        let sq1 = squares.get(1).unwrap();
        let pinned = pos.get_piece_at(sq0);
        let attacker = pos.get_piece_at(sq1);

        // pinned piece must be of the same color as the king
        // and the attacked piece must be of the opposite color
        if !(pinned.color() == color && attacker.color() == color.opponent()) {
            continue;
        }

        let pinned = match attacker.get_type() {
            PieceType::Queen => true,
            PieceType::Rook => i < 4, // Rook moves in 0-3 directions
            PieceType::Bishop => i >= 4 && i < 8, // Bishop moves in 4-7 directions
            _ => false,
        };
        if pinned {
            // If the pinned piece is a rook or bishop, add the pin mask
            pin_map |= sq0.to_bitboard();
        }
    }

    pin_map
}

fn move_mask_rook(sq: Square, my_occupancy: BitBoard, enemy_occupancy: BitBoard) -> BitBoard {
    move_mask_sliding::<0, 4>(sq, my_occupancy, enemy_occupancy)
}

fn move_mask_bishop(sq: Square, my_occupancy: BitBoard, enemy_occupancy: BitBoard) -> BitBoard {
    move_mask_sliding::<4, 8>(sq, my_occupancy, enemy_occupancy)
}

fn move_mask_queen(sq: Square, my_occupancy: BitBoard, enemy_occupancy: BitBoard) -> BitBoard {
    move_mask_sliding::<0, 8>(sq, my_occupancy, enemy_occupancy)
}

fn pseudo_legal_move_general(move_list: &mut MoveList, sq: Square, move_mask: BitBoard) {
    let mut bb = move_mask;
    while bb.any() {
        let target_sq = bb.first_nonzero_sq();
        move_list.add(Move::new(sq, target_sq, MoveType::Normal, None));
        bb.remove_first_nonzero_sq();
    }
}

fn pseudo_legal_move_rook(
    move_list: &mut MoveList,
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) {
    let mask = move_mask_rook(sq, my_occupancy, enemy_occupancy);
    pseudo_legal_move_general(move_list, sq, mask);
}

fn pseudo_legal_move_bishop(
    move_list: &mut MoveList,
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) {
    let mask = move_mask_bishop(sq, my_occupancy, enemy_occupancy);
    pseudo_legal_move_general(move_list, sq, mask);
}

fn pseudo_legal_move_queen(
    move_list: &mut MoveList,
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) {
    let mask = move_mask_queen(sq, my_occupancy, enemy_occupancy);
    pseudo_legal_move_general(move_list, sq, mask);
}
/* #endregion */

/* #region */
/// Computes the legal knight moves from a given square on a bitboard.
///
/// The knight moves in an L-shape: two squares in one cardinal direction
/// (N, S, E, W) followed by one square in a perpendicular direction.
///
/// ## Movement Description
///
/// A knight on a square can move in 8 possible directions:
///
/// - NE + N (2N 1E) → North-North-East
/// - NW + N (2N 1W) → North-North-West
/// - SE + S (2S 1E) → South-South-East
/// - SW + S (2S 1W) → South-South-West
/// - NW + W (2W 1N) → West-West-North
/// - SW + W (2W 1S) → West-West-South
/// - NE + E (2E 1N) → East-East-North
/// - SE + E (2E 1S) → East-East-South

fn move_mask_knight(sq: Square, my_occupancy: BitBoard) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();

    let mask = !my_occupancy;

    moves |= shift(bb & !(BOUND_H | BOUND_78), NE + NORTH) & mask;
    moves |= shift(bb & !(BOUND_A | BOUND_78), NW + NORTH) & mask;
    moves |= shift(bb & !(BOUND_H | BOUND_12), SE + SOUTH) & mask;
    moves |= shift(bb & !(BOUND_A | BOUND_12), SW + SOUTH) & mask;

    moves |= shift(bb & !(BOUND_AB | BOUND_8), NW + WEST) & mask;
    moves |= shift(bb & !(BOUND_AB | BOUND_1), SW + WEST) & mask;
    moves |= shift(bb & !(BOUND_GH | BOUND_8), NE + EAST) & mask;
    moves |= shift(bb & !(BOUND_GH | BOUND_1), SE + EAST) & mask;

    moves
}

fn pseudo_legal_move_knight(move_list: &mut MoveList, sq: Square, my_occupancy: BitBoard) {
    let mask = move_mask_knight(sq, my_occupancy);
    pseudo_legal_move_general(move_list, sq, mask);
}
/* #endregion */

/* #region */
/// Computes the legal moves for a king from a given square on a bitboard,
/// including standard single-square moves and optional castling.
///
/// The king can move one square in any direction (horizontal, vertical, diagonal),
/// and under special conditions, can perform a castling move with a rook.
///
/// ## Movement Directions
///
/// - N  → North
/// - S  → South
/// - E  → East
/// - W  → West
/// - NE → North-East
/// - NW → North-West
/// - SE → South-East
/// - SW → South-West
///
/// ## Standard Movement
///
/// The king normally moves to any adjacent square (up to 8 directions), provided
/// the destination is not occupied by a friendly piece and is not under attack.
///
/// ## Castling
///
/// Castling is a special move involving the king and one of the rooks. There are
/// two types of castling:
///
/// - Kingside Castling:
///   - White: King moves from E1 to G1, rook from H1 to F1
///   - Black: King moves from E8 to G8, rook from H8 to F8
///
/// - Queenside Castling:
///   - White: King moves from E1 to C1, rook from A1 to D1
///   - Black: King moves from E8 to C8, rook from A8 to D8
///
/// ### Castling Conditions
///
/// Castling is only legal if all the following are true:
/// - Neither the king nor the involved rook has previously moved
/// - The squares between the king and rook are empty
/// - The king is not currently in check
/// - The king does not pass through or land on a square that is under attack

fn move_mask_king<const COLOR: u8, const ATTACK_ONLY: bool>(
    sq: Square,
    pos: &Position,
) -> BitBoard {
    let color = Color::from(COLOR);
    let is_white = color.is_white();

    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();
    let occupancy = !pos.occupancies[COLOR as usize];
    moves |= shift_north(bb) & occupancy;
    moves |= shift_south(bb) & occupancy;
    moves |= shift_east(bb) & occupancy;
    moves |= shift_west(bb) & occupancy;
    moves |= shift_ne(bb) & occupancy;
    moves |= shift_nw(bb) & occupancy;
    moves |= shift_se(bb) & occupancy;
    moves |= shift_sw(bb) & occupancy;

    if !ATTACK_ONLY {
        // If we are checking if cells are being attacked, not actually moving, no need exclude pieces under attack
        moves &= !pos.attack_map_color[color.opponent().as_usize()];

        if is_white {
            if (pos.castling & MoveFlags::K != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::G1, Square::H1)
            {
                assert!(sq == Square::E1);
                moves.set_sq(Square::G1);
            }
            if (pos.castling & MoveFlags::Q != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::C1, Square::A1)
            {
                assert!(sq == Square::E1);
                moves.set_sq(Square::C1);
            }
        } else {
            if (pos.castling & MoveFlags::k != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::G8, Square::H8)
            {
                assert!(sq == Square::E8);
                moves.set_sq(Square::G8);
            }
            if (pos.castling & MoveFlags::q != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::C8, Square::A8)
            {
                assert!(sq == Square::E8);
                moves.set_sq(Square::C8);
            }
        }
    }

    moves
}

fn move_mask_castle_check<const COLOR: u8>(
    pos: &Position,
    sq: Square,
    dst_sq: Square,
    rook_sq: Square,
) -> bool {
    // r . . . k . . r
    // a b c d e f g h
    let color = Color::from(COLOR);
    let opponent = color.opponent();

    // check if the rook is in the right place
    let rook_type = Piece::get_piece(color, PieceType::Rook);
    if pos.bitboards[rook_type.as_usize()].test(rook_sq.as_u8()) == false {
        return false;
    }

    fn min_max(a: Square, b: Square) -> (Square, Square) {
        if a.0 < b.0 { (a, b) } else { (b, a) }
    }

    // check if the cells are under attack
    let (start, end) = min_max(sq, dst_sq);
    for i in start.0..=end.0 {
        if pos.attack_map_color[opponent.as_usize()].test(i) {
            return false;
        }
    }

    // check if any piece is in the way
    let (start, end) = min_max(sq, rook_sq);
    for i in start.0 + 1..end.0 {
        if (pos.occupancies[Color::BOTH.as_usize()]).test(i) {
            return false;
        }
    }

    true
}

pub fn pseudo_legal_move_king<const COLOR: u8>(
    move_list: &mut MoveList,
    sq: Square,
    pos: &Position,
) {
    let mut mask = move_mask_king::<COLOR, false>(sq, pos);
    while mask.any() {
        let target_sq = mask.first_nonzero_sq();

        // check if it's a castling move
        let (from_file, _) = sq.file_rank();
        let (to_file, _) = target_sq.file_rank();
        let diff = from_file as i8 - to_file as i8;
        let move_type = match diff.abs() {
            0 | 1 => MoveType::Normal,
            2 => MoveType::Castling,
            _ => panic!("Invalid castling move from {} to {}", sq, target_sq),
        };

        move_list.add(Move::new(sq, target_sq, move_type, None));
        mask.remove_first_nonzero_sq();
    }
}

/* #endregion */

#[cfg(test)]
mod tests {
    use super::*;

    pub fn pseudo_legal_move_from(pos: &Position, sq: Square) -> BitBoard {
        pseudo_legal_from_sq_impl::<false>(pos, sq, pos.side_to_move)
    }

    fn squares_to_bitboard(sqs: &[Square]) -> BitBoard {
        let mut bb = BitBoard::new();
        for &sq in sqs {
            bb.set_sq(sq);
        }
        bb
    }

    const BB_D4: BitBoard = Square::D4.to_bitboard();
    const BB_D5: BitBoard = Square::D5.to_bitboard();
    const BB_D6: BitBoard = Square::D6.to_bitboard();

    const BB_E3: BitBoard = Square::E3.to_bitboard();
    const BB_E4: BitBoard = Square::E4.to_bitboard();
    const BB_E5: BitBoard = Square::E5.to_bitboard();

    #[test]
    fn test_white_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );

        let moves = pseudo_legal_move_from(&pos, Square::E2);
        assert_eq!(moves, BB_E3 | BB_E4);
    }

    #[test]
    fn test_black_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::D7);
        assert_eq!(moves, BB_D6 | BB_D5);
    }

    #[test]
    fn white_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E4);
        assert_eq!(moves, BB_D5 | BB_E5);
    }

    #[test]
    fn black_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::D5);
        assert_eq!(moves, BB_D4 | BB_E4);
    }

    #[test]
    fn move_biship() {
        let fen = "rn1qkbnr/ppp1pppp/4b3/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E6);
        assert_eq!(
            moves,
            squares_to_bitboard(&[Square::C8, Square::D7, Square::F5, Square::G4, Square::H3])
        );
    }

    #[test]
    fn move_rook() {
        let fen = "rnbqkbn1/ppp1pp1r/7p/3p2p1/7P/3P1PPR/PPP1P3/RNBQKBN1 b - - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::H7);
        assert_eq!(moves, squares_to_bitboard(&[Square::H8, Square::G7]));
    }

    #[test]
    fn move_knight() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::B1);
        assert_eq!(moves, squares_to_bitboard(&[Square::A3, Square::C3]));
    }

    #[test]
    fn knight_attack() {
        let fen = "rn2kb1r/pppqppp1/5n2/3p3p/4P1b1/BPN5/P1PP1PPP/R2QKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::F6);
        assert_eq!(moves, squares_to_bitboard(&[Square::E4, Square::G8, Square::H7]));
    }

    #[test]
    fn test_castling() {
        let fen = "r3k2r/ppp1bppp/2n1pn2/3p4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b kq - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E8);
        assert_eq!(
            moves,
            squares_to_bitboard(&[Square::C8, Square::D8, Square::F8, Square::G8, Square::D7,])
        );
    }

    #[test]
    fn test_castling_2() {
        let fen = "8/4k3/8/8/8/8/r6R/R3K3 w Q - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E1);
        assert!(moves.test(Square::C1.as_u8()))
    }

    #[test]
    fn test_pin() {
        // 2 . . . . . . . k
        // 1 K B . . . . . r
        //   a b c d e f g h
        let pos = Position::from("8/8/8/8/8/8/7k/KB5r w - - 0 1").unwrap();

        assert_eq!(
            pos.attack_map_color[Color::BLACK.as_usize()],
            BitBoard::from(0b11000000_01000000_01111110)
        );

        let is_pinned = pos.is_square_pinned(Square::B1, Color::WHITE);

        assert!(is_pinned, "Move bishop to A2 exposes king to check");
    }

    #[test]
    fn test_rook_pin_pawn() {
        let pos = Position::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

        let is_pinned = pos.is_square_pinned(Square::B5, Color::WHITE);

        assert!(is_pinned, "Pawn B5 is pinned by rook on H5");
    }

    #[test]
    fn test_en_passant() {
        let pos = Position::from("4k3/8/8/4Pp2/8/8/8/4K3 w - f6 2 4").unwrap();

        // 8 . . . . k . . .
        // 7 . . . . . . . .
        // 6 . . . . . . . .
        // 5 . . . . P p . .
        // 4 . . . . . . . .
        // 3 . . . . . . . .
        // 2 . . . . . . . .
        // 1 . . . . K . . .
        //   a b c d e f g h

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, 0);
        assert_eq!(pos.en_passant.unwrap(), Square::F6);
        assert_eq!(pos.halfmove_clock, 2);
        assert_eq!(pos.fullmove_number, 4);
        assert_eq!(
            pos.to_board_string(),
            "....k.......................Pp..............................K..."
        );

        let moves = pseudo_legal_move_from(&pos, Square::E5);
        assert_eq!(moves, Square::E6.to_bitboard() | Square::F6.to_bitboard());
    }

    #[test]
    fn capture_resolve_check() {
        let mut pos = Position::from(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();

        let m = Move::new(Square::B2, Square::B3, MoveType::Normal, None);
        assert!(is_pseudo_move_legal(&mut pos, &m));
        pos.make_move(m);

        let m = Move::new(Square::C5, Square::E3, MoveType::Normal, None);
        assert!(is_pseudo_move_legal(&mut pos, &m));
        pos.make_move(m);

        let m = Move::new(Square::F2, Square::E3, MoveType::Normal, None);
        let legal = is_pseudo_move_legal(&mut pos, &m);
        assert!(legal, "Capture on E3 should resolve check");
    }

    #[test]
    fn en_passant_expose_check() {
        let mut pos = Position::from("8/8/8/KP5r/1R3p1k/8/4P3/8 w - - 0 1").unwrap();
        let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
        assert!(is_pseudo_move_legal(&mut pos, &m), "E2 to E4 should be legal");
        pos.make_move(m);

        let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
        assert!(
            !is_pseudo_move_legal(&mut pos, &m),
            "En passant is illegal because it exposes king to check"
        );
    }

    #[test]
    fn en_passant_capture_checker() {
        let mut pos = Position::from("8/8/8/KP1k3r/1R3p2/8/4P3/8 w - - 0 1").unwrap();
        let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
        assert!(is_pseudo_move_legal(&mut pos, &m), "E2 to E4 should be legal");
        pos.make_move(m);

        let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
        assert!(
            is_pseudo_move_legal(&mut pos, &m),
            "En passant is illegal it captures the attacking pawn"
        );
    }
}

/* #region */

const NORTH: i32 = 8;
const SOUTH: i32 = -NORTH;
const EAST: i32 = 1;
const WEST: i32 = -EAST;
const NE: i32 = NORTH + EAST;
const NW: i32 = NORTH + WEST;
const SE: i32 = SOUTH + EAST;
const SW: i32 = SOUTH + WEST;

const BOUND_A: BitBoard = BitBoard::from(0x0101010101010101);
const BOUND_B: BitBoard = BitBoard::from(0x0202020202020202);
const BOUND_G: BitBoard = BitBoard::from(0x4040404040404040);
const BOUND_H: BitBoard = BitBoard::from(0x8080808080808080);
const BOUND_1: BitBoard = BitBoard::from(0x00000000000000FF);
const BOUND_2: BitBoard = BitBoard::from(0x000000000000FF00);
const BOUND_7: BitBoard = BitBoard::from(0x00FF000000000000);
const BOUND_8: BitBoard = BitBoard::from(0xFF00000000000000);
const BOUND_AB: BitBoard = BitBoard::from(BOUND_A.get() | BOUND_B.get());
const BOUND_GH: BitBoard = BitBoard::from(BOUND_G.get() | BOUND_H.get());
const BOUND_12: BitBoard = BitBoard::from(BOUND_1.get() | BOUND_2.get());
const BOUND_78: BitBoard = BitBoard::from(BOUND_7.get() | BOUND_8.get());

fn shift(bb: BitBoard, dir: i32) -> BitBoard {
    // if dir > 0 { bb.get() << dir } else { bb.get() >> -dir }
    BitBoard::from(if dir < 0 { bb.get() >> -dir } else { bb.get() << dir })
}

fn shift_east(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_H).shift(EAST)
}

fn shift_west(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_A).shift(WEST)
}

fn shift_north(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_8).shift(NORTH)
}

fn shift_south(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_1).shift(SOUTH)
}

fn shift_ne(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_H | BOUND_8)).shift(NE)
}

fn shift_nw(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_A | BOUND_8)).shift(NW)
}

fn shift_se(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_H | BOUND_1)).shift(SE)
}

fn shift_sw(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_A | BOUND_1)).shift(SW)
}

const SHIFT_FUNCS: [fn(BitBoard) -> BitBoard; 8] =
    [shift_north, shift_south, shift_east, shift_west, shift_ne, shift_nw, shift_se, shift_sw];

/* #endregion */
