use super::super::position::{Position, SmallSquareList};
use super::super::types::*;
use super::internal::*;

/// Generates all pseudo-legal moves for a given piece on a specific square.
///
/// This function appends moves to the provided `move_list` based on the piece type,
/// current board `pos`, and the starting square `sq`. Pseudo-legal moves include
/// all moves the piece *could* make ignoring checks.
///
/// # Arguments
///
/// * `pos` - The current board position.
/// * `sq` - The square from which the piece moves.
/// * `piece` - The chess piece for which to generate moves.
/// * `move_list` - Mutable reference to a list where generated moves will be stored.
///
/// # Note
///
/// Moves are *pseudo-legal*, so no validation is done regarding king safety.
pub fn pseudo_legal_moves_src_sq(
    pos: &Position,
    sq: Square,
    piece: Piece,
    move_list: &mut MoveList,
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

/// Computes the attack mask for a given piece on a specific square.
///
/// Returns a `BitBoard` representing all squares attacked by the piece from `sq`
/// on the current board position `pos`. Unlike move generation, the attack mask
/// includes squares occupied by friendly pieces since attacks consider threats,
/// not legal moves.
///
/// # Arguments
///
/// * `pos` - The current board position.
/// * `sq` - The square where the piece is located.
/// * `piece` - The type of piece whose attacks are being calculated.
///
/// # Returns
///
/// A `BitBoard` with bits set for each square attacked by the piece.
pub fn attack_mask_src_sq(pos: &Position, sq: Square, piece: Piece) -> BitBoard {
    let color = piece.color();

    let my_occupancy = pos.occupancies[color.as_usize()];
    let enemy_occupancy = pos.occupancies[color.opponent().as_usize()];

    match piece {
        Piece::W_PAWN => pawn_mask::<{ Color::WHITE.as_u8() }, true>(sq, pos),
        Piece::B_PAWN => pawn_mask::<{ Color::BLACK.as_u8() }, true>(sq, pos),
        Piece::W_ROOK | Piece::B_ROOK => rook_mask::<true>(sq, my_occupancy, enemy_occupancy),
        Piece::W_BISHOP | Piece::B_BISHOP => bishop_mask::<true>(sq, my_occupancy, enemy_occupancy),
        Piece::W_QUEEN | Piece::B_QUEEN => queen_mask::<true>(sq, my_occupancy, enemy_occupancy),
        Piece::W_KNIGHT | Piece::B_KNIGHT => knight_mask::<true>(sq, my_occupancy),
        Piece::W_KING => king_mask::<{ Color::WHITE.as_u8() }, true>(sq, pos),
        Piece::B_KING => king_mask::<{ Color::BLACK.as_u8() }, true>(sq, pos),
        Piece::NONE => BitBoard::new(),
        _ => {
            panic!("Invalid piece type: {:?}", piece);
        }
    }
}

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

fn pawn_mask<const COLOR: u8, const ATTACK_MASK: bool>(sq: Square, pos: &Position) -> BitBoard {
    let (_file, rank) = sq.file_rank();
    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();

    let opponent = COLOR ^ 1;

    let is_white = COLOR == Color::WHITE.as_u8();
    let is_black = COLOR != Color::WHITE.as_u8();

    if !ATTACK_MASK {
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
    if ATTACK_MASK || (attack_left & pos.occupancies[opponent as usize]).any() {
        moves |= attack_left;
    }
    let attack_right = if is_white { shift_ne(bb) } else { shift_se(bb) };
    if ATTACK_MASK || (attack_right & pos.occupancies[opponent as usize]).any() {
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
    let mask = pawn_mask::<{ COLOR }, false>(sq, pos);

    for dst_sq in mask.iter() {
        if check_if_promotion::<COLOR>(dst_sq) {
            // Promotion move
            let promotion_types =
                [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight];
            for &promotion in &promotion_types {
                move_list.add(Move::new(sq, dst_sq, MoveType::Promotion, Some(promotion)));
            }
        } else {
            let is_ep_capture =
                check_if_eq_capture::<COLOR>(pos, sq, dst_sq, pos.get_piece_at(dst_sq));
            let move_type = if is_ep_capture { MoveType::EnPassant } else { MoveType::Normal };
            move_list.add(Move::new(sq, dst_sq, move_type, None));
        }
    }
}

fn check_if_promotion<const COLOR: u8>(dst_sq: Square) -> bool {
    let (_, rank) = dst_sq.file_rank();

    match rank {
        RANK_8 if COLOR == Color::WHITE.as_u8() => true,
        RANK_1 if COLOR == Color::BLACK.as_u8() => true,
        _ => false,
    }
}

fn check_if_eq_capture<const COLOR: u8>(
    pos: &Position,
    src_sq: Square,
    dst_sq: Square,
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
    let (from_file, from_rank) = src_sq.file_rank();
    let (to_file, to_rank) = dst_sq.file_rank();
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
fn sliding_mask<const START: u8, const END: u8, const ATTACK_MASK: bool>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
    // color: Color,
) -> BitBoard {
    let mut masks = BitBoard::new();
    let bb = sq.to_bitboard();

    for i in START..END {
        let mut next_bb = SHIFT_FUNCS[i as usize](bb);

        while next_bb.any() {
            if (next_bb & my_occupancy).any() {
                if ATTACK_MASK {
                    masks |= next_bb;
                }
                break;
            }

            if (next_bb & enemy_occupancy).any() {
                masks |= next_bb;
                break;
            }

            masks |= next_bb;

            next_bb = SHIFT_FUNCS[i as usize](next_bb);
        }
    }

    masks
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

fn rook_mask<const ATTACK_MASK: bool>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) -> BitBoard {
    sliding_mask::<0, 4, ATTACK_MASK>(sq, my_occupancy, enemy_occupancy)
}

fn bishop_mask<const ATTACK_MASK: bool>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) -> BitBoard {
    sliding_mask::<4, 8, ATTACK_MASK>(sq, my_occupancy, enemy_occupancy)
}

fn queen_mask<const ATTACK_MASK: bool>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) -> BitBoard {
    sliding_mask::<0, 8, ATTACK_MASK>(sq, my_occupancy, enemy_occupancy)
}

fn pseudo_legal_move_general(move_list: &mut MoveList, sq: Square, move_mask: BitBoard) {
    for dst_sq in move_mask.iter() {
        move_list.add(Move::new(sq, dst_sq, MoveType::Normal, None));
    }
}

fn pseudo_legal_move_rook(
    move_list: &mut MoveList,
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) {
    let mask = rook_mask::<false>(sq, my_occupancy, enemy_occupancy);
    pseudo_legal_move_general(move_list, sq, mask);
}

fn pseudo_legal_move_bishop(
    move_list: &mut MoveList,
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) {
    let mask = bishop_mask::<false>(sq, my_occupancy, enemy_occupancy);
    pseudo_legal_move_general(move_list, sq, mask);
}

fn pseudo_legal_move_queen(
    move_list: &mut MoveList,
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) {
    let mask = queen_mask::<false>(sq, my_occupancy, enemy_occupancy);
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

fn knight_mask<const ATTACK_MASK: bool>(sq: Square, my_occupancy: BitBoard) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();

    let mask = if ATTACK_MASK { 0u64 } else { my_occupancy.get() };
    let mask = BitBoard::from(!mask);

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
    let mask = knight_mask::<false>(sq, my_occupancy);
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

fn king_mask<const COLOR: u8, const ATTACK_MASK: bool>(sq: Square, pos: &Position) -> BitBoard {
    let color = Color::from(COLOR);
    let is_white = color.is_white();

    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();
    let occupancy = !(if ATTACK_MASK { BitBoard::new() } else { pos.occupancies[COLOR as usize] });
    moves |= shift_north(bb) & occupancy;
    moves |= shift_south(bb) & occupancy;
    moves |= shift_east(bb) & occupancy;
    moves |= shift_west(bb) & occupancy;
    moves |= shift_ne(bb) & occupancy;
    moves |= shift_nw(bb) & occupancy;
    moves |= shift_se(bb) & occupancy;
    moves |= shift_sw(bb) & occupancy;

    if !ATTACK_MASK {
        // If we are checking if cells are being attacked, not actually moving, no need exclude pieces under attack
        moves &= !pos.attack_mask[color.opponent().as_usize()];

        if is_white {
            if (pos.castling & CastlingRight::K != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::G1, Square::H1)
            {
                assert!(sq == Square::E1);
                moves.set_sq(Square::G1);
            }
            if (pos.castling & CastlingRight::Q != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::C1, Square::A1)
            {
                assert!(sq == Square::E1);
                moves.set_sq(Square::C1);
            }
        } else {
            if (pos.castling & CastlingRight::k != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::G8, Square::H8)
            {
                assert!(sq == Square::E8);
                moves.set_sq(Square::G8);
            }
            if (pos.castling & CastlingRight::q != 0)
                && move_mask_castle_check::<COLOR>(pos, sq, Square::C8, Square::A8)
            {
                assert!(sq == Square::E8);
                moves.set_sq(Square::C8);
            }
        }
    }

    moves
}

/// @TODO: refactor this
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
    let (s1, e1) = min_max(sq, dst_sq);
    let (s2, e2) = min_max(sq, rook_sq);

    let checks = [
        (s1.0, e1.0 + 1, pos.attack_mask[opponent.as_usize()]),
        (s2.0 + 1, e2.0, pos.occupancies[Color::BOTH.as_usize()]),
    ];

    for (start, end, mask) in checks {
        for i in start..end {
            if mask.test(i) {
                return false;
            }
        }
    }

    true
}

pub fn pseudo_legal_move_king<const COLOR: u8>(
    move_list: &mut MoveList,
    sq: Square,
    pos: &Position,
) {
    let mask = king_mask::<COLOR, false>(sq, pos);
    for target_sq in mask.iter() {
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
    }
}

/* #endregion */
