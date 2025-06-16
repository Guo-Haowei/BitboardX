use super::UndoState;
use crate::engine::position::{
    CastlingRight, CastlingType, Move, MoveType, Piece, Position, Square,
};
use crate::engine::types::{
    BitBoard, Color, FILE_A, FILE_H, PieceType, RANK_2, RANK_4, RANK_5, RANK_7,
};

pub fn make_move(pos: &mut Position, m: Move) -> UndoState {
    let src_sq = m.from_sq();
    let dst_sq = m.to_sq();
    let src_piece = pos.get_piece_at(src_sq);
    let dst_piece = pos.get_piece_at(dst_sq);
    let src_type = src_piece.get_type();
    let src_piece_idx = src_piece.as_usize();
    let mover_type = m.get_type();
    let mover_color = src_piece.color();
    debug_assert!(src_piece != Piece::NONE, "No piece found on 'from' square");
    debug_assert!(pos.side_to_move == mover_color, "Trying to move a piece of the wrong color");

    // Copy undo state before making changes to the position
    let undo_state = UndoState {
        castling: pos.castling,
        en_passant: pos.en_passant,
        halfmove_clock: pos.halfmove_clock,
        fullmove_number: pos.fullmove_number,
        dst_piece,
    };

    let disabled_castling = match src_type {
        PieceType::King | PieceType::Rook => {
            drop_castling(pos, src_sq, dst_sq, src_piece, dst_piece)
        }
        _ => 0,
    };

    // -------------- Update Board Start --------------
    do_move_ep(pos, m, src_piece);

    debug_assert!(pos.occupancies[pos.side_to_move.as_usize()].test(src_sq.as_u8()));

    move_piece(&mut pos.bitboards[src_piece_idx], src_sq, dst_sq);

    let captured_something = if dst_piece != Piece::NONE {
        // Clear the 'to' square for the captured piece
        pos.bitboards[dst_piece.as_usize()].unset(dst_sq.as_u8());
        true
    } else {
        false
    };

    // special move handling
    match mover_type {
        MoveType::Castling => {
            // Castling move, we need to move the king and rook
            debug_assert!(src_type == PieceType::King, "Castling must be a king move");
            debug_assert!(dst_piece == Piece::NONE, "Castling must not capture any piece");

            // king already moved to the destination square, only need to move the rook
            let index = castling_type(src_piece, src_sq, dst_sq);
            assert!(index != CastlingType::None, "Invalid castling move");
            // move rook position
            let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
            move_piece(&mut pos.bitboards[piece.as_usize()], from_sq, to_sq);
        }
        MoveType::Promotion => {
            debug_assert!(src_type == PieceType::Pawn);
            let promotion = Piece::get_piece(mover_color, m.get_promotion().unwrap());
            pos.bitboards[src_piece_idx].unset(dst_sq.as_u8()); // Remove the pawn from the board
            pos.bitboards[promotion.as_usize()].set(dst_sq.as_u8()); // Place the promoted piece on the board
        }
        _ => {}
    }

    // -------------- Update Board End --------------

    pos.castling &= !disabled_castling;
    pos.en_passant = update_en_passant_square(pos, m.from_sq(), m.to_sq(), src_piece);
    pos.fullmove_number += if mover_color == Color::WHITE { 0 } else { 1 };

    if captured_something || src_piece.get_type() != PieceType::Pawn {
        pos.halfmove_clock = 0; // reset halfmove clock if a piece was captured or a non-pawn moved
    } else {
        pos.halfmove_clock += 1; // increment halfmove clock for a pawn move
    }

    pos.post_move();

    undo_state
}

pub fn unmake_move(pos: &mut Position, m: Move, undo_state: &UndoState) {
    let from = pos.get_piece_at(m.to_sq());

    undo_move_generic(pos, m, from, undo_state.dst_piece);

    undo_promotion(pos, m);

    undo_move_ep(pos, m, from);

    undo_castling(pos, m, from);

    pos.post_move();

    pos.restore(undo_state);
}

fn do_move_ep(pos: &mut Position, m: Move, src_piece: Piece) {
    let (to_file, _) = m.to_sq().file_rank();

    // check if it's an en passant capture
    if m.get_type() == MoveType::EnPassant {
        // capture the opponent's pawn passed en passant square
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(src_piece.color().opponent(), PieceType::Pawn);

        debug_assert!(pos.get_piece_at(enemy_sq) == enemy);
        debug_assert!(pos.get_piece_at(m.to_sq()) == Piece::NONE);

        // Remove the captured pawn from the board
        pos.bitboards[enemy.as_usize()].unset(enemy_sq.as_u8());
    }
}

fn undo_move_ep(pos: &mut Position, m: Move, from: Piece) {
    if m.get_type() == MoveType::EnPassant {
        // Restore the captured pawn on the en passant square
        let (to_file, _) = m.to_sq().file_rank();
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(from.color().opponent(), PieceType::Pawn);

        debug_assert!(pos.get_piece_at(enemy_sq) == Piece::NONE);

        // Place the captured pawn back on the board
        pos.bitboards[enemy.as_usize()].set(enemy_sq.as_u8());
    }
}

fn move_piece(board: &mut BitBoard, src_sq: Square, dst_sq: Square) {
    debug_assert!(board.test(src_sq.as_u8()), "No piece found on 'from' square");
    board.unset(src_sq.as_u8());
    board.set(dst_sq.as_u8());
}

fn undo_move_generic(pos: &mut Position, m: Move, from: Piece, to: Piece) {
    let from_sq = m.from_sq();
    let to_sq = m.to_sq();

    move_piece(&mut pos.bitboards[from.as_usize()], to_sq, from_sq);

    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].set(m.to_sq().as_u8()); // Place captured piece back on 'to' square
    }
}

const CASTLING_ROOK_SQUARES: [(Piece, Square, Square); 4] = [
    (Piece::W_ROOK, Square::H1, Square::F1), // White King-side
    (Piece::W_ROOK, Square::A1, Square::D1), // White Queen-side
    (Piece::B_ROOK, Square::H8, Square::F8), // Black King-side
    (Piece::B_ROOK, Square::A8, Square::D8), // Black Queen-side
];

fn castling_type(from: Piece, from_sq: Square, to_sq: Square) -> CastlingType {
    match (from, from_sq, to_sq) {
        (Piece::W_KING, Square::E1, Square::G1) => CastlingType::WhiteKingSide,
        (Piece::W_KING, Square::E1, Square::C1) => CastlingType::WhiteQueenSide,
        (Piece::B_KING, Square::E8, Square::G8) => CastlingType::BlackKingSide,
        (Piece::B_KING, Square::E8, Square::C8) => CastlingType::BlackQueenSide,
        _ => CastlingType::None,
    }
}

fn undo_promotion(pos: &mut Position, m: Move) {
    if m.get_type() != MoveType::Promotion {
        return;
    }

    // from square is the square of the promoted piece
    let from_sq = m.from_sq();
    let piece = pos.get_piece_at(from_sq);
    let color = piece.color();
    let promotion = Piece::get_piece(color, m.get_promotion().unwrap());
    let pawn = Piece::get_piece(color, PieceType::Pawn);

    pos.bitboards[pawn.as_usize()].set(from_sq.as_u8()); // Place the pawn back on the board
    pos.bitboards[promotion.as_usize()].unset(from_sq.as_u8()); // Remove the promoted piece from the board
}

fn undo_castling(pos: &mut Position, m: Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == CastlingType::None {
        return;
    }

    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], to_sq, from_sq);
}

// if castling rights are already disabled, return
// if king moved, disable castling rights, return
// if rook moved, disable castling rights, return
// if rook taken out, disable castling rights, return
fn drop_castling(
    pos: &Position,
    src_sq: Square,
    dst_sq: Square,
    src_piece: Piece,
    dst_piece: Piece,
) -> u8 {
    debug_assert!(
        src_piece.get_type() == PieceType::King || src_piece.get_type() == PieceType::Rook
    );

    fn helper<const TEST_BIT: u8>(
        pos: &Position,
        src_sq: Square,
        dst_sq: Square,
        src_piece: Piece,
        dst_piece: Piece,
    ) -> u8 {
        if pos.castling & TEST_BIT == 0 {
            return 0;
        }

        if src_piece == Piece::W_KING && (TEST_BIT & CastlingRight::KQ) != 0 {
            return TEST_BIT;
        }

        if src_piece == Piece::B_KING && (TEST_BIT & CastlingRight::kq) != 0 {
            return TEST_BIT;
        }

        match (src_piece, src_sq) {
            (Piece::W_ROOK, Square::A1) if (TEST_BIT & CastlingRight::Q) != 0 => return TEST_BIT,
            (Piece::W_ROOK, Square::H1) if (TEST_BIT & CastlingRight::K) != 0 => return TEST_BIT,
            (Piece::B_ROOK, Square::A8) if (TEST_BIT & CastlingRight::q) != 0 => return TEST_BIT,
            (Piece::B_ROOK, Square::H8) if (TEST_BIT & CastlingRight::k) != 0 => return TEST_BIT,
            _ => {}
        }

        match (dst_piece, dst_sq) {
            (Piece::W_ROOK, Square::A1) if (TEST_BIT & CastlingRight::Q) != 0 => return TEST_BIT,
            (Piece::W_ROOK, Square::H1) if (TEST_BIT & CastlingRight::K) != 0 => return TEST_BIT,
            (Piece::B_ROOK, Square::A8) if (TEST_BIT & CastlingRight::q) != 0 => return TEST_BIT,
            (Piece::B_ROOK, Square::H8) if (TEST_BIT & CastlingRight::k) != 0 => return TEST_BIT,
            _ => {}
        }

        0
    }

    let mut mask = 0;
    mask |= helper::<{ CastlingRight::K }>(pos, src_sq, dst_sq, src_piece, dst_piece);
    mask |= helper::<{ CastlingRight::Q }>(pos, src_sq, dst_sq, src_piece, dst_piece);
    mask |= helper::<{ CastlingRight::k }>(pos, src_sq, dst_sq, src_piece, dst_piece);
    mask |= helper::<{ CastlingRight::q }>(pos, src_sq, dst_sq, src_piece, dst_piece);
    mask
}

fn update_en_passant_square(
    pos: &Position,
    from_sq: Square,
    to_sq: Square,
    from: Piece,
) -> Option<Square> {
    if from.get_type() != PieceType::Pawn {
        return None;
    }

    let (file, from_rank) = from_sq.file_rank();
    let (_file, to_rank) = to_sq.file_rank();

    if match (from, from_rank, to_rank) {
        (Piece::W_PAWN, RANK_2, RANK_4) => true,
        (Piece::B_PAWN, RANK_7, RANK_5) => true,
        _ => false,
    } {
        assert_eq!(file, _file);
        // check if there's opponent's pawn on the left or right of 'to' square
        let board = &pos.bitboards[if from == Piece::W_PAWN {
            Piece::B_PAWN.as_usize()
        } else {
            Piece::W_PAWN.as_usize()
        }];

        if (file < FILE_H && board.test(Square::make(file + 1, to_rank).as_u8()))
            || (file > FILE_A && board.test(Square::make(file - 1, to_rank).as_u8()))
        {
            return Some(Square::make(file, (from_rank + to_rank) / 2));
        }
    }

    None
}
