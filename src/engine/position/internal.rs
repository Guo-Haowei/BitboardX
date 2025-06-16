use super::UndoState;
use crate::engine::position::{Castling, Move, MoveFlags, MoveType, Piece, Position, Square};
use crate::engine::types::{
    BitBoard, Color, FILE_A, FILE_H, PieceType, RANK_2, RANK_4, RANK_5, RANK_7,
};

pub fn make_move(pos: &mut Position, m: Move) -> UndoState {
    let from = pos.get_piece_at(m.from_sq());
    let from_sq = m.from_sq();
    let to_sq = m.to_sq();
    let to_piece = pos.get_piece_at(to_sq);

    // Copy undo state before making changes to the position
    let undo_state = UndoState {
        castling: pos.castling,
        en_passant: pos.en_passant,
        halfmove_clock: pos.halfmove_clock,
        fullmove_number: pos.fullmove_number,
        to_piece,
    };

    let disabled_castling = drop_castling(
        pos,
        from_sq,
        to_sq,
        pos.get_piece_at(m.from_sq()),
        pos.get_piece_at(m.to_sq()),
    );

    do_move_ep(pos, m, from);

    debug_assert!(pos.occupancies[pos.side_to_move.as_usize()].test(m.from_sq().as_u8()));

    move_piece(&mut pos.bitboards[from.as_usize()], from_sq, to_sq);

    if to_piece != Piece::NONE {
        pos.bitboards[to_piece.as_usize()].unset(m.to_sq().as_u8()); // Clear the 'to' square for the captured piece
    }

    do_castling(pos, m, from);

    do_promotion(pos, m, from);

    pos.castling &= !disabled_castling;
    pos.en_passant = update_en_passant_square(pos, m.from_sq(), m.to_sq(), from);
    pos.fullmove_number += if pos.side_to_move == Color::WHITE { 0 } else { 1 };

    pos.post_move();

    undo_state
}

pub fn unmake_move(pos: &mut Position, m: Move, undo_state: &UndoState) {
    let from = pos.get_piece_at(m.to_sq());

    undo_move_generic(pos, m, from, undo_state.to_piece);

    undo_promotion(pos, m);

    undo_move_ep(pos, m, from);

    undo_castling(pos, m, from);

    pos.post_move();

    pos.restore(undo_state);
}

fn do_move_ep(pos: &mut Position, m: Move, from: Piece) {
    let (to_file, _) = m.to_sq().file_rank();

    // check if it's an en passant capture
    if m.get_type() == MoveType::EnPassant {
        // capture the opponent's pawn passed en passant square
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(from.color().opponent(), PieceType::Pawn);

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

fn move_piece(board: &mut BitBoard, from_sq: Square, to_sq: Square) {
    debug_assert!(board.test(from_sq.as_u8()), "No piece found on 'from' square");
    board.unset(from_sq.as_u8());
    board.set(to_sq.as_u8());
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

fn castling_type(from: Piece, from_sq: Square, to_sq: Square) -> Castling {
    match (from, from_sq, to_sq) {
        (Piece::W_KING, Square::E1, Square::G1) => Castling::WhiteKingSide,
        (Piece::W_KING, Square::E1, Square::C1) => Castling::WhiteQueenSide,
        (Piece::B_KING, Square::E8, Square::G8) => Castling::BlackKingSide,
        (Piece::B_KING, Square::E8, Square::C8) => Castling::BlackQueenSide,
        _ => Castling::None,
    }
}

fn do_promotion(pos: &mut Position, m: Move, from: Piece) {
    if m.get_type() != MoveType::Promotion {
        return;
    }

    assert!(
        from.get_type() == PieceType::Pawn,
        "Promotion must be from a pawm, got '{}'",
        from.to_char()
    );

    let color = from.color();
    let to_sq = m.to_sq();
    let promotion = Piece::get_piece(color, m.get_promotion().unwrap());

    pos.bitboards[from.as_usize()].unset(to_sq.as_u8()); // Remove the pawn from the board
    pos.bitboards[promotion.as_usize()].set(to_sq.as_u8()); // Place the promoted piece on the board
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

fn do_castling(pos: &mut Position, m: Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }
    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], from_sq, to_sq);
}

fn undo_castling(pos: &mut Position, m: Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }

    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], to_sq, from_sq);
}

// if castling rights are already disabled, return
// if king moved, disable castling rights, return
// if rook moved, disable castling rights, return
// if rook taken out, disable castling rights, return
fn drop_castling(pos: &Position, from_sq: Square, to_sq: Square, from: Piece, to: Piece) -> u8 {
    fn helper<const BIT: u8>(
        pos: &Position,
        from_sq: Square,
        to_sq: Square,
        from: Piece,
        to: Piece,
    ) -> u8 {
        if pos.castling & BIT == 0 {
            return 0;
        }

        if from == Piece::W_KING && (BIT & MoveFlags::KQ) != 0 {
            return BIT;
        }

        if from == Piece::B_KING && (BIT & MoveFlags::kq) != 0 {
            return BIT;
        }

        match (from, from_sq) {
            (Piece::W_ROOK, Square::A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
            (Piece::W_ROOK, Square::H1) if (BIT & MoveFlags::K) != 0 => return BIT,
            (Piece::B_ROOK, Square::A8) if (BIT & MoveFlags::q) != 0 => return BIT,
            (Piece::B_ROOK, Square::H8) if (BIT & MoveFlags::k) != 0 => return BIT,
            _ => {}
        }

        match (to, to_sq) {
            (Piece::W_ROOK, Square::A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
            (Piece::W_ROOK, Square::H1) if (BIT & MoveFlags::K) != 0 => return BIT,
            (Piece::B_ROOK, Square::A8) if (BIT & MoveFlags::q) != 0 => return BIT,
            (Piece::B_ROOK, Square::H8) if (BIT & MoveFlags::k) != 0 => return BIT,
            _ => {}
        }

        0
    }

    let mut drop_castling = 0;
    drop_castling |= helper::<{ MoveFlags::K }>(pos, from_sq, to_sq, from, to);
    drop_castling |= helper::<{ MoveFlags::Q }>(pos, from_sq, to_sq, from, to);
    drop_castling |= helper::<{ MoveFlags::k }>(pos, from_sq, to_sq, from, to);
    drop_castling |= helper::<{ MoveFlags::q }>(pos, from_sq, to_sq, from, to);
    drop_castling
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
