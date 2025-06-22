use super::UndoState;
use crate::core::position::*;

// Assume passed in moves are legal
pub fn make_move(_pos: &mut Position, mv: Move) -> UndoState {
    // borrow the position immutably because we are just checking the move at this point, no actual moving
    let pos: &Position = _pos;

    let src_sq = mv.src_sq();
    let dst_sq = mv.dst_sq();
    let src_piece = pos.get_piece_at(src_sq);
    let dst_piece = pos.get_piece_at(dst_sq);
    let src_piece_type = src_piece.get_type();
    let src_piece_idx = src_piece.as_usize();
    let move_type = mv.get_type();
    let mover_color = src_piece.color();
    let enemy_color = mover_color.opponent();
    let is_mover_pawn = src_piece_type == PieceType::PAWN;
    let enemy_pawn = Piece::get_piece(enemy_color, PieceType::PAWN);
    let (src_file, src_rank) = src_sq.file_rank();
    let (dst_file, dst_rank) = dst_sq.file_rank();

    debug_assert!(src_piece != Piece::NONE, "No piece found on 'from' square");
    debug_assert!(
        pos.state.side_to_move == mover_color,
        "Trying to move a piece of the wrong color"
    );

    // check if the move will change the castling rights
    let castling_rights =
        castling_right_mask(pos.state.castling_rights, src_sq, dst_sq, src_piece, dst_piece);

    // check if the move will generate an en passant square
    let mut en_passant_sq: Option<Square> = None;
    loop {
        if !is_mover_pawn {
            break;
        }
        let dy = dst_rank.diff(src_rank).abs();
        debug_assert!(dy <= 2, "Pawn move must be 1 or 2 squares");
        if dy == 1 {
            break;
        }
        let enemy_pawns = pos.bitboards[enemy_pawn.as_usize()];
        let dst_sq_bb = dst_sq.to_bitboard();
        let east = dst_sq_bb.shift_east();
        let west = dst_sq_bb.shift_west();

        // if there's an enemy pawn on the east or west square, we can generate an en passant square
        if (east & enemy_pawns).any() || (west & enemy_pawns).any() {
            en_passant_sq = Some(Square::make(src_file, Rank((src_rank.0 + dst_rank.0) / 2)));
        }

        break;
    }

    // -------------- Update Board Start --------------
    // rebind pos to the mutable reference to update the position
    let pos = _pos;
    pos.state.captured_piece = dst_piece;
    let undo_state = pos.state;

    debug_assert!(pos.state.occupancies[pos.state.side_to_move.as_usize()].test(src_sq.as_u8()));

    move_piece(&mut pos.bitboards[src_piece_idx], src_sq, dst_sq);

    let captured_something = if dst_piece != Piece::NONE {
        // Clear the 'to' square for the captured piece
        pos.bitboards[dst_piece.as_usize()].unset(dst_sq.as_u8());
        true
    } else {
        false
    };

    // special move handling
    match move_type {
        MoveType::Castling => {
            // Castling move, we need to move the king and rook
            debug_assert!(src_piece_type == PieceType::KING, "Castling must be a king move");
            debug_assert!(dst_piece == Piece::NONE, "Castling must not capture any piece");

            // king already moved to the destination square, only need to move the rook
            let index = castling_type(src_piece, src_sq, dst_sq);
            assert!(index != CastlingType::None, "Invalid castling move");
            // move rook position
            let (piece, src_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
            move_piece(&mut pos.bitboards[piece.as_usize()], src_sq, to_sq);
        }
        MoveType::Promotion => {
            assert!(src_piece_type == PieceType::PAWN);
            let promotion = Piece::get_piece(mover_color, mv.get_promotion().unwrap());
            pos.bitboards[src_piece_idx].unset(dst_sq.as_u8()); // Remove the pawn from the board
            pos.bitboards[promotion.as_usize()].set(dst_sq.as_u8()); // Place the promoted piece on the board
        }
        MoveType::EnPassant => {
            assert!(src_piece_type == PieceType::PAWN, "En passant must be a pawn move");
            let enemy_sq = Square::make(dst_file, src_rank);
            let enemy = Piece::get_piece(enemy_color, PieceType::PAWN);

            debug_assert!(pos.get_piece_at(enemy_sq) == enemy);
            debug_assert!(
                pos.get_piece_at(dst_sq) == src_piece,
                "attacking pawn is already moved to the destination square at this point"
            );
            pos.bitboards[enemy.as_usize()].unset(enemy_sq.as_u8());
        }
        _ => {}
    }

    // -------------- Update Board End --------------

    pos.state.side_to_move = pos.state.side_to_move.opponent();
    pos.state.castling_rights = castling_rights;
    pos.state.en_passant = en_passant_sq;
    pos.state.fullmove_number += if mover_color == Color::WHITE { 0 } else { 1 };

    if captured_something || is_mover_pawn {
        pos.state.halfmove_clock = 0; // reset halfmove clock if a piece was captured or a non-pawn moved
    } else {
        pos.state.halfmove_clock += 1; // increment halfmove clock for a pawn move
    }

    update_cache(pos);

    undo_state
}

pub fn unmake_move(pos: &mut Position, mv: Move, undo_state: &UndoState) {
    // Keep in mind that the move is already applied to the position
    let src_sq = mv.src_sq();
    let dst_sq = mv.dst_sq();
    let src_piece = pos.get_piece_at(dst_sq); // the src_piece is the piece that was moved to the dst_sq
    let captured_piece = undo_state.captured_piece;
    let mover_color = src_piece.color();
    let enemy_color = mover_color.opponent();
    let enemy_pawn = Piece::get_piece(enemy_color, PieceType::PAWN);

    move_piece(&mut pos.bitboards[src_piece.as_usize()], dst_sq, src_sq);

    if captured_piece != Piece::NONE {
        pos.bitboards[captured_piece.as_usize()].set(dst_sq.as_u8()); // Place captured piece back on 'to' square
    }

    match mv.get_type() {
        MoveType::Castling => {
            debug_assert!(src_piece.get_type() == PieceType::KING);
            // Restore Rook position
            let index = castling_type(src_piece, src_sq, dst_sq);
            debug_assert!(index != CastlingType::None);

            let (piece, src_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
            move_piece(&mut pos.bitboards[piece.as_usize()], to_sq, src_sq);
        }
        MoveType::Promotion => {
            let promotion = Piece::get_piece(mover_color, mv.get_promotion().unwrap());
            let our_pawn = Piece::get_piece(mover_color, PieceType::PAWN);

            pos.bitboards[our_pawn.as_usize()].set(src_sq.as_u8()); // Place the pawn back on the board
            pos.bitboards[promotion.as_usize()].unset(src_sq.as_u8()); // Remove the promoted piece from the board
        }
        MoveType::EnPassant => {
            // en passant is special, because if there's a captured piece, it will be placed on the wrong square
            // debug_assert!(undo_state.captured_piece == enemy_pawn);
            let (_, from_rank) = src_sq.file_rank();
            let (to_file, _) = dst_sq.file_rank();
            let enemy_sq = Square::make(to_file, from_rank);

            debug_assert!(pos.get_piece_at(enemy_sq) == Piece::NONE);

            pos.bitboards[enemy_pawn.as_usize()].set(enemy_sq.as_u8());
        }
        _ => {}
    }

    pos.state = *undo_state;
}

pub fn update_cache(pos: &mut Position) {
    // update occupancies
    pos.state.occupancies[Color::WHITE.as_usize()] = pos.bitboards[Piece::W_PAWN.as_usize()]
        | pos.bitboards[Piece::W_KNIGHT.as_usize()]
        | pos.bitboards[Piece::W_BISHOP.as_usize()]
        | pos.bitboards[Piece::W_ROOK.as_usize()]
        | pos.bitboards[Piece::W_QUEEN.as_usize()]
        | pos.bitboards[Piece::W_KING.as_usize()];
    pos.state.occupancies[Color::BLACK.as_usize()] = pos.bitboards[Piece::B_PAWN.as_usize()]
        | pos.bitboards[Piece::B_KNIGHT.as_usize()]
        | pos.bitboards[Piece::B_BISHOP.as_usize()]
        | pos.bitboards[Piece::B_ROOK.as_usize()]
        | pos.bitboards[Piece::B_QUEEN.as_usize()]
        | pos.bitboards[Piece::B_KING.as_usize()];
    pos.state.occupancies[Color::BOTH.as_usize()] = pos.state.occupancies[Color::WHITE.as_usize()]
        | pos.state.occupancies[Color::BLACK.as_usize()];

    // update attack maps
    pos.update_attack_map_and_checker();

    // maybe only need to update the side to move attack map?
    pos.state.pin_map[Color::WHITE.as_usize()] = move_gen::generate_pin_map(pos, Color::WHITE);
    pos.state.pin_map[Color::BLACK.as_usize()] = move_gen::generate_pin_map(pos, Color::BLACK);
}

fn move_piece(board: &mut BitBoard, from_sq: Square, to_sq: Square) {
    debug_assert!(board.test(from_sq.as_u8()), "No piece found on 'from' square");
    board.unset(from_sq.as_u8());
    board.set(to_sq.as_u8());
}

const CASTLING_ROOK_SQUARES: [(Piece, Square, Square); 4] = [
    (Piece::W_ROOK, Square::H1, Square::F1), // White King-side
    (Piece::W_ROOK, Square::A1, Square::D1), // White Queen-side
    (Piece::B_ROOK, Square::H8, Square::F8), // Black King-side
    (Piece::B_ROOK, Square::A8, Square::D8), // Black Queen-side
];

fn castling_type(src_piece: Piece, src_sq: Square, dst_sq: Square) -> CastlingType {
    match (src_piece, src_sq, dst_sq) {
        (Piece::W_KING, Square::E1, Square::G1) => CastlingType::WhiteKingSide,
        (Piece::W_KING, Square::E1, Square::C1) => CastlingType::WhiteQueenSide,
        (Piece::B_KING, Square::E8, Square::G8) => CastlingType::BlackKingSide,
        (Piece::B_KING, Square::E8, Square::C8) => CastlingType::BlackQueenSide,
        _ => CastlingType::None,
    }
}

// if castling rights are already disabled, return
// if king moved, disable castling rights, return
// if rook moved, disable castling rights, return
// if rook taken out, disable castling rights, return
fn castling_right_mask(
    old_flags: u8,
    src_sq: Square,
    dst_sq: Square,
    src_piece: Piece,
    dst_piece: Piece,
) -> u8 {
    fn lost_castle_right_at_bit(
        castling_type: CastlingType,
        src_sq: Square,
        dst_sq: Square,
        src_piece: Piece,
        dst_piece: Piece,
    ) -> bool {
        let index = castling_type as usize;
        let mask = 1u8 << index;
        let (rook_piece, rook_sq, _) = CASTLING_ROOK_SQUARES[index];

        // if the rook is moving away, the castling rights are disabled
        if src_piece == rook_piece && src_sq == rook_sq {
            return true;
        }

        // if the rook was captured, the castling rights are disabled
        if dst_piece == rook_piece && dst_sq == rook_sq {
            return true;
        }

        match src_piece {
            Piece::W_KING if (mask & CastlingRight::KQ) != 0 => true,
            Piece::B_KING if (mask & CastlingRight::kq) != 0 => true,
            _ => false,
        }
    }

    let mut new_flags = old_flags;
    for i in 0..4 {
        let castling_type = unsafe { std::mem::transmute::<u8, CastlingType>(i as u8) };
        let bitmask = 1u8 << i;
        if new_flags & bitmask == 0 {
            continue; // If the right already lost, skip
        }
        if lost_castle_right_at_bit(castling_type, src_sq, dst_sq, src_piece, dst_piece) {
            new_flags &= !bitmask; // Disable the castling right if it can't be performed
        }
    }

    new_flags
}
