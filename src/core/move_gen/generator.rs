use crate::core::position::{CheckerList, Position, SmallSquareList};
use crate::core::types::bitboard::*;
use crate::core::types::*;

// @TODO: get rid of this evntually, use magic bitboards instead
const SHIFT_FUNCS: [fn(&BitBoard) -> BitBoard; 8] = [
    BitBoard::shift_north,
    BitBoard::shift_south,
    BitBoard::shift_east,
    BitBoard::shift_west,
    BitBoard::shift_ne,
    BitBoard::shift_nw,
    BitBoard::shift_se,
    BitBoard::shift_sw,
];

const MV_MASK_MOVE: u8 = 0;
const MV_MASK_CAPTURE: u8 = 1;
const MV_MASK_ATTACK: u8 = 2;

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
pub fn pseudo_legal_moves_src_sq<const MASK: u8>(
    pos: &Position,
    sq: Square,
    piece: Piece,
    list: &mut MoveList,
) {
    let color = piece.color();

    let my = pos.state.occupancies[color.as_usize()];
    let enemy = pos.state.occupancies[color.flip().as_usize()];

    if piece == Piece::W_PAWN {
        return pseudo_legal_move_pawn::<0, MASK>(list, sq, pos);
    }
    if piece == Piece::B_PAWN {
        return pseudo_legal_move_pawn::<1, MASK>(list, sq, pos);
    }

    let mask = match piece {
        Piece::W_KNIGHT | Piece::B_KNIGHT => knight_mask::<MASK>(sq, my, enemy),
        Piece::W_ROOK | Piece::B_ROOK => rook_mask::<MASK>(sq, my, enemy),
        Piece::W_BISHOP | Piece::B_BISHOP => bishop_mask::<MASK>(sq, my, enemy),
        Piece::W_QUEEN | Piece::B_QUEEN => queen_mask::<MASK>(sq, my, enemy),
        _ => panic!("Invalid piece type: {:?}", piece),
    };

    pseudo_legal_move_general(list, sq, mask);
}

/// Pseudo-legal move generation
pub fn pseudo_legal_moves(pos: &Position) -> MoveList {
    pseudo_legal_moves_impl::<MV_MASK_MOVE>(pos)
}

pub fn pseudo_legal_capture_moves(pos: &Position) -> MoveList {
    pseudo_legal_moves_impl::<MV_MASK_CAPTURE>(pos)
}

fn pseudo_legal_moves_impl<const MASK: u8>(pos: &Position) -> MoveList {
    let mut move_list = MoveList::new();

    let color = pos.side_to_move;
    let king_sq = pos.get_king_square(color);
    let (start, end) = if color == Color::WHITE {
        pseudo_legal_move_king::<0, MASK>(&mut move_list, king_sq, pos);
        (Piece::W_START, Piece::W_END)
    } else {
        pseudo_legal_move_king::<1, MASK>(&mut move_list, king_sq, pos);
        (Piece::B_START, Piece::B_END)
    };

    // early return if double check
    if pos.state.checkers[color.as_usize()].count() == 2 {
        return move_list;
    }

    for i in start..end {
        let piece = Piece::new(i);

        for sq in pos.bitboards[i as usize].iter() {
            pseudo_legal_moves_src_sq::<MASK>(pos, sq, piece, &mut move_list);
        }
    }

    move_list
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

fn pawn_mask<const COLOR: u8, const FILTER_TYPE: u8>(sq: Square, pos: &Position) -> BitBoard {
    let opponent = COLOR ^ 1;
    let is_white = COLOR == 0;

    let attack_mask = PAWM_ATTACK_MASKS[COLOR as usize][sq.as_usize()];

    if FILTER_TYPE == MV_MASK_ATTACK {
        return attack_mask;
    }

    let attacks = attack_mask & pos.state.occupancies[opponent as usize];
    if FILTER_TYPE == MV_MASK_CAPTURE {
        return attacks;
    }

    let offset = if is_white { 8i8 } else { -8i8 };
    let rank = if is_white { BitBoard::MASK_4 } else { BitBoard::MASK_5 };
    let empty_mask = !pos.state.occupancies[2].get();
    let mut single_push_mask = 1u64 << (sq.as_u8() as i8 + offset);
    single_push_mask &= empty_mask;
    // if can't single push, then single push is 0,
    // double push mask will never land on rank 4 or 5
    let mut double_push_mask = if is_white { single_push_mask << 8 } else { single_push_mask >> 8 };
    double_push_mask &= empty_mask & rank;

    BitBoard::from(single_push_mask | double_push_mask) | attacks
}

fn pseudo_legal_move_pawn<const COLOR: u8, const FILTER_TYPE: u8>(
    move_list: &mut MoveList,
    sq: Square,
    pos: &Position,
) {
    let mask = pawn_mask::<COLOR, FILTER_TYPE>(sq, pos);

    for dst_sq in mask.iter() {
        let sq_mask = 1u64 << dst_sq.as_u8();
        let promo_rank = if COLOR == 0 { BitBoard::MASK_8 } else { BitBoard::MASK_1 };
        if sq_mask & promo_rank != 0 {
            // Promotion move
            let promotion_types =
                [PieceType::QUEEN, PieceType::ROOK, PieceType::BISHOP, PieceType::KNIGHT];
            for &promotion in &promotion_types {
                move_list.add(Move::new(sq, dst_sq, MoveType::Promotion, Some(promotion)));
            }
        } else {
            move_list.add(Move::new(sq, dst_sq, MoveType::Normal, None));
        }
    }

    if let Some(ep_sq) = pos.state.en_passant {
        let attack_mask = pawn_mask::<{ COLOR }, MV_MASK_ATTACK>(sq, pos);
        // if attach mask and ep square overlap, then it's an en passant capture
        if attack_mask.test_sq(ep_sq) {
            move_list.add(Move::new(sq, ep_sq, MoveType::EnPassant, None));
        }
    }
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
fn sliding_mask<const START: u8, const END: u8, const MASK: u8>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
    // color: Color,
) -> BitBoard {
    let mut masks = BitBoard::new();
    let bb = sq.to_bitboard();

    let any_occupied = my_occupancy | enemy_occupancy;

    for i in START..END {
        let mut next_bb = SHIFT_FUNCS[i as usize](&bb);

        while next_bb.any() {
            if (next_bb & any_occupied).any() {
                masks |= next_bb;
                break;
            }

            masks |= next_bb;

            next_bb = SHIFT_FUNCS[i as usize](&next_bb);
        }
    }

    match MASK {
        MV_MASK_MOVE => masks & !my_occupancy,
        MV_MASK_ATTACK => masks,
        MV_MASK_CAPTURE => masks & enemy_occupancy,
        _ => panic!("Invalid mask type: {}", MASK),
    }
}

pub fn generate_pin_map(pos: &Position, color: Color) -> BitBoard {
    let mut pin_map = BitBoard::new();

    let occupied = pos.state.occupancies[Color::BOTH.as_usize()];
    let king_bb = pos.bitboards[Piece::get_piece(color, PieceType::KING).as_usize()];
    debug_assert!(king_bb.count() == 1, "There must be exactly one king on the board");

    for i in 0..8 {
        let mut next_bb = SHIFT_FUNCS[i as usize](&king_bb);

        let mut squares = SmallSquareList::new();

        loop {
            let next_sq = match next_bb.to_square() {
                Some(sq) => sq,
                None => break, // No more squares in this direction
            };
            if occupied.test_sq(next_sq) {
                squares.add(next_sq);
                if squares.count() == 2 {
                    break; // Found two pieces in this direction
                }
            }
            next_bb = SHIFT_FUNCS[i as usize](&next_bb);
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
        if !(pinned.color() == color && attacker.color() == color.flip()) {
            continue;
        }

        let pinned = match attacker.get_type() {
            PieceType::QUEEN => true,
            PieceType::ROOK => i < 4, // Rook moves in 0-3 directions
            PieceType::BISHOP => i >= 4 && i < 8, // Bishop moves in 4-7 directions
            _ => false,
        };
        if pinned {
            // If the pinned piece is a rook or bishop, add the pin mask
            pin_map |= sq0.to_bitboard();
        }
    }

    pin_map
}

fn rook_mask<const MASK: u8>(sq: Square, mine: BitBoard, enemy: BitBoard) -> BitBoard {
    sliding_mask::<0, 4, MASK>(sq, mine, enemy)
}

fn bishop_mask<const MASK: u8>(sq: Square, mine: BitBoard, enemy: BitBoard) -> BitBoard {
    sliding_mask::<4, 8, MASK>(sq, mine, enemy)
}

fn queen_mask<const MASK: u8>(sq: Square, mine: BitBoard, enemy: BitBoard) -> BitBoard {
    sliding_mask::<0, 8, MASK>(sq, mine, enemy)
}

fn pseudo_legal_move_general(move_list: &mut MoveList, sq: Square, move_mask: BitBoard) {
    for dst_sq in move_mask.iter() {
        move_list.add(Move::new(sq, dst_sq, MoveType::Normal, None));
    }
}

/* #endregion */

/* #region */
/// Computes the legal knight moves from a given square on a bitboard.
/// use precomputed knight attack masks for efficiency.
fn knight_mask<const MASK: u8>(
    sq: Square,
    my_occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) -> BitBoard {
    let mut mask = KNIGHT_MASKS[sq.as_usize()];
    if MASK == MV_MASK_ATTACK {
        return mask;
    }
    mask &= !my_occupancy;
    if MASK == MV_MASK_MOVE {
        return mask;
    }
    mask &= enemy_occupancy;
    mask
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

fn king_mask<const COLOR: u8, const MASK_TYPE: u8>(sq: Square, pos: &Position) -> BitBoard {
    let color = Color::new(COLOR);
    let is_white = color.is_white();

    let mut moves = KING_MASKS[sq.as_usize()];

    if MASK_TYPE == MV_MASK_ATTACK {
        return moves;
    }

    // if it's move mask, remove squares occupied by own pieces
    moves &= !pos.state.occupancies[COLOR as usize];
    // and remove squares being attacked by the opponent
    moves &= !pos.state.attack_mask[color.flip().as_usize()];

    if MASK_TYPE == MV_MASK_CAPTURE {
        return moves & pos.state.occupancies[(COLOR ^ 1) as usize];
    }

    // check castling possibilities
    let offset = if is_white { 0 } else { 2 };

    for i in 0..2 {
        let bit = i + offset;
        let flag = 1u8 << bit;
        if flag & pos.state.castling_rights != 0 {
            let path_clear = (KING_CASTLING_CLEAR_MASK[bit]
                & pos.state.occupancies[Color::BOTH.as_usize()])
            .none();
            let path_safe = (KING_CASTLING_SAFE_MASKS[bit]
                & pos.state.attack_mask[(COLOR ^ 1) as usize])
                .none();
            let rook_still_there = pos.bitboards
                [Piece::get_piece(color, PieceType::ROOK).as_usize()]
            .test_sq(KING_CASTLING_ROOK_SQ[bit]);
            if path_clear && path_safe && rook_still_there {
                moves.set_sq(KING_CASTLING_DEST_SQ[bit]);
            }
        }
    }

    moves
}

pub fn pseudo_legal_move_king<const COLOR: u8, const MASK_TYPE: u8>(
    move_list: &mut MoveList,
    sq: Square,
    pos: &Position,
) {
    let mask = king_mask::<COLOR, MASK_TYPE>(sq, pos);
    for dst_sq in mask.iter() {
        // check if it's a castling move
        let (src_file, _) = sq.file_rank();
        let (dst_file, _) = dst_sq.file_rank();
        let diff = src_file.diff(dst_file);
        let move_type = match diff.abs() {
            0 | 1 => MoveType::Normal,
            2 => MoveType::Castling,
            _ => panic!("Invalid castling move from {} to {}", sq, dst_sq),
        };

        move_list.add(Move::new(sq, dst_sq, move_type, None));
    }
}

/* #endregion */

pub fn calc_attack_map_and_checker<const COLOR: u8>(pos: &mut Position) -> (BitBoard, CheckerList) {
    let enemy_king = if COLOR == 0 { Piece::B_KING } else { Piece::W_KING };
    let enemy_king_mask = pos.bitboards[enemy_king.as_usize()];

    let pawn = if COLOR == 0 { Piece::W_PAWN } else { Piece::B_PAWN };
    let knight = if COLOR == 0 { Piece::W_KNIGHT } else { Piece::B_KNIGHT };
    let bishop = if COLOR == 0 { Piece::W_BISHOP } else { Piece::B_BISHOP };
    let rook = if COLOR == 0 { Piece::W_ROOK } else { Piece::B_ROOK };
    let queen = if COLOR == 0 { Piece::W_QUEEN } else { Piece::B_QUEEN };
    let king = if COLOR == 0 { Piece::W_KING } else { Piece::B_KING };

    let my_occupancy = pos.state.occupancies[COLOR as usize];
    let enemy_occupancy = pos.state.occupancies[(COLOR ^ 1) as usize];

    let mut masks = [BitBoard::new(); 6];

    let mut checkers = CheckerList::new();

    const MASK: u8 = MV_MASK_ATTACK;

    // pawn
    let mask = &mut masks[PieceType::PAWN.as_usize()];
    for sq in pos.bitboards[pawn.as_usize()].iter() {
        let sq_attack_mask = pawn_mask::<{ COLOR }, MASK>(sq, pos);
        *mask |= sq_attack_mask;
        if (sq_attack_mask & enemy_king_mask).any() {
            checkers.add(sq);
        }
    }

    // knight
    let mask = &mut masks[PieceType::KNIGHT.as_usize()];
    for sq in pos.bitboards[knight.as_usize()].iter() {
        let sq_attack_mask = knight_mask::<MASK>(sq, my_occupancy, enemy_occupancy);
        *mask |= sq_attack_mask;
        if (sq_attack_mask & enemy_king_mask).any() {
            checkers.add(sq);
        }
    }

    // bishop
    let mask = &mut masks[PieceType::BISHOP.as_usize()];
    for sq in pos.bitboards[bishop.as_usize()].iter() {
        let sq_attack_mask = bishop_mask::<MASK>(sq, my_occupancy, enemy_occupancy);
        *mask |= sq_attack_mask;
        if (sq_attack_mask & enemy_king_mask).any() {
            checkers.add(sq);
        }
    }

    // rook
    let mask = &mut masks[PieceType::ROOK.as_usize()];
    for sq in pos.bitboards[rook.as_usize()].iter() {
        let sq_attack_mask = rook_mask::<MASK>(sq, my_occupancy, enemy_occupancy);
        *mask |= sq_attack_mask;
        if (sq_attack_mask & enemy_king_mask).any() {
            checkers.add(sq);
        }
    }

    // queen
    let mask = &mut masks[PieceType::QUEEN.as_usize()];
    for sq in pos.bitboards[queen.as_usize()].iter() {
        let sq_attack_mask = queen_mask::<MASK>(sq, my_occupancy, enemy_occupancy);
        *mask |= sq_attack_mask;
        if (sq_attack_mask & enemy_king_mask).any() {
            checkers.add(sq);
        }
    }

    // king
    let mask = &mut masks[PieceType::KING.as_usize()];
    for sq in pos.bitboards[king.as_usize()].iter() {
        *mask |= king_mask::<{ COLOR }, MASK>(sq, pos);
        // not possible to check the king with king, so no need to check for overlap
    }

    let mut final_mask = BitBoard::new();
    for mask in masks.iter().copied() {
        final_mask |= mask;
    }

    (final_mask, checkers)
}

/// Precomputes the move masks
const fn build_pawn_attack_mask<const IS_WHITE: bool>(file: u8, rank: u8) -> BitBoard {
    let mut mask = BitBoard::new();

    if rank == 0 || rank == 7 {
        return mask; // No pawn moves on the first or last rank
    }
    if IS_WHITE {
        if file > 0 {
            mask.set_sq(Square::make(File(file - 1), Rank(rank + 1)));
        }
        if file < 7 {
            mask.set_sq(Square::make(File(file + 1), Rank(rank + 1)));
        }
    } else {
        if file > 0 {
            mask.set_sq(Square::make(File(file - 1), Rank(rank - 1)));
        }
        if file < 7 {
            mask.set_sq(Square::make(File(file + 1), Rank(rank - 1)));
        }
    }

    mask
}

const fn build_pawn_attack_masks() -> [[BitBoard; 64]; 2] {
    let mut masks = [[BitBoard::new(); 64]; 2];

    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;

        masks[0][sq] = build_pawn_attack_mask::<true>(file as u8, rank as u8); // White pawns
        masks[1][sq] = build_pawn_attack_mask::<false>(file as u8, rank as u8); // Black pawns

        sq += 1;
    }

    masks
}

const fn build_pawn_en_passant_mask(file: u8, rank: u8) -> BitBoard {
    let mut mask = BitBoard::new();

    if file > 0 {
        mask.set_sq(Square::make(File(file - 1), Rank(rank)));
    }
    if file < 7 {
        mask.set_sq(Square::make(File(file + 1), Rank(rank)));
    }

    mask
}

const fn build_knight_mask(file: u8, rank: u8) -> BitBoard {
    let mut mask = BitBoard::new();

    const OFFSETS: [(i8, i8); 8] =
        [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];

    let mut idx = 0;
    while idx < OFFSETS.len() {
        let (df, dr) = OFFSETS[idx];
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;

        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            mask.set_sq(Square::make(File(new_file as u8), Rank(new_rank as u8)));
        }
        idx += 1;
    }

    mask
}

const fn build_king_mask(file: u8, rank: u8) -> BitBoard {
    let mut mask = BitBoard::new();

    const OFFSETS: [(i8, i8); 8] =
        [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

    let mut idx = 0;
    while idx < OFFSETS.len() {
        let (df, dr) = OFFSETS[idx];
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;

        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            mask.set_sq(Square::make(File(new_file as u8), Rank(new_rank as u8)));
        }
        idx += 1;
    }

    mask
}

const PAWM_ATTACK_MASKS: [[BitBoard; 64]; 2] = build_pawn_attack_masks();

pub const PAWN_EN_PASSANT_MASKS: [[BitBoard; 64]; 2] = {
    let mut masks = [[BitBoard::new(); 64]; 2];
    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;
        if rank == 3 {
            masks[0][sq] = build_pawn_en_passant_mask(file as u8, 3);
        }
        if rank == 4 {
            masks[1][sq] = build_pawn_en_passant_mask(file as u8, 4);
        }
        sq += 1;
    }
    masks
};

const KNIGHT_MASKS: [BitBoard; 64] = {
    let mut masks = [BitBoard::new(); 64];
    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;
        masks[sq] = build_knight_mask(file as u8, rank as u8);
        sq += 1;
    }
    masks
};

const KING_MASKS: [BitBoard; 64] = {
    let mut masks = [BitBoard::new(); 64];
    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;
        masks[sq] = build_king_mask(file as u8, rank as u8);
        sq += 1;
    }
    masks
};

const B1_MASK: u64 = 1u64 << Square::B1.as_u8();
const C1_MASK: u64 = 1u64 << Square::C1.as_u8();
const D1_MASK: u64 = 1u64 << Square::D1.as_u8();
const E1_MASK: u64 = 1u64 << Square::E1.as_u8();
const F1_MASK: u64 = 1u64 << Square::F1.as_u8();
const G1_MASK: u64 = 1u64 << Square::G1.as_u8();

const B8_MASK: u64 = 1u64 << Square::B8.as_u8();
const C8_MASK: u64 = 1u64 << Square::C8.as_u8();
const D8_MASK: u64 = 1u64 << Square::D8.as_u8();
const E8_MASK: u64 = 1u64 << Square::E8.as_u8();
const F8_MASK: u64 = 1u64 << Square::F8.as_u8();
const G8_MASK: u64 = 1u64 << Square::G8.as_u8();

const KING_CASTLING_CLEAR_MASK: [BitBoard; 4] = [
    BitBoard::from(F1_MASK | G1_MASK),           // White kingside
    BitBoard::from(B1_MASK | C1_MASK | D1_MASK), // White queenside
    BitBoard::from(F8_MASK | G8_MASK),           // Black kingside
    BitBoard::from(B8_MASK | C8_MASK | D8_MASK), // Black queenside
];

const KING_CASTLING_SAFE_MASKS: [BitBoard; 4] = [
    BitBoard::from(E1_MASK | F1_MASK | G1_MASK), // White kingside
    BitBoard::from(C1_MASK | D1_MASK | E1_MASK), // White queenside
    BitBoard::from(E8_MASK | F8_MASK | G8_MASK), // Black kingside
    BitBoard::from(C8_MASK | D8_MASK | E8_MASK), // Black queenside
];

const KING_CASTLING_DEST_SQ: [Square; 4] = [
    Square::G1, // White kingside
    Square::C1, // White queenside
    Square::G8, // Black kingside
    Square::C8, // Black queenside
];

const KING_CASTLING_ROOK_SQ: [Square; 4] = [
    Square::H1, // White kingside
    Square::A1, // White queenside
    Square::H8, // Black kingside
    Square::A8, // Black queenside
];
