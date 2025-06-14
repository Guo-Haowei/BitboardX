use crate::engine::position::Snapshot;

use super::board::{BitBoard, Square, constants::*};
use super::piece::*;
use super::position::Position;
use modular_bitfield::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Castling {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
    None,
}

pub struct MoveFlags;

#[allow(non_upper_case_globals)]
impl MoveFlags {
    pub const K: u8 = 1u8 << Castling::WhiteKingSide as u8;
    pub const Q: u8 = 1u8 << Castling::WhiteQueenSide as u8;
    pub const k: u8 = 1u8 << Castling::BlackKingSide as u8;
    pub const q: u8 = 1u8 << Castling::BlackQueenSide as u8;
    pub const KQ: u8 = Self::K | Self::Q;
    pub const kq: u8 = Self::k | Self::q;
    pub const KQkq: u8 = Self::KQ | Self::kq;
}

#[repr(u8)]
pub enum SpecialMove {
    None = 0,
    Castling = 1,
    EnPassant = 2,
    Promotion = 3,
}

/// Flags:
///   1. 6 bits to store the from square (0-63).
///   2. 6 bits to store the to square (0-63).
///   3. 1 bit to store if this move introduces an en passant square,
///      if the bit is on, we can retreive the square starting square).
///   4. 1 bit to store if this move drops an en passant square,
///      and extra 8 bits to store which file it is.
///   5. 1 bits to store if this move is an en passant capture.
///   6. 2 bits to store castling right drop, 1 bit for each side.
///   7. 2 bits to store the promotion piece type (Queen, Rook, Bishop, Knight).

/// 0     6    12       14       15     16
/// |- 6 -|- 6 -|-- 4 ---|-- 1 --|-- 1 --|- 2 --|- 4 --|
///   from  to     castle    color   ep_mv

#[bitfield]
#[derive(Debug, Clone, Copy)]
struct PackedData {
    from_sq: B6,
    to_sq: B6,
    is_ep_capture: B1, // if this move captures enemy pawn by en passant rule
    #[skip]
    __: B19,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    data: PackedData,
}

impl Move {
    pub fn new(from_sq: Square, to_sq: Square, from: Piece, is_ep_capture: bool) -> Self {
        debug_assert!(from != Piece::NONE);
        let mut data = PackedData::new();
        data.set_from_sq(from_sq.as_u8());
        data.set_to_sq(to_sq.as_u8());
        data.set_is_ep_capture(is_ep_capture as u8);

        Self { data }
    }

    fn from_sq(&self) -> Square {
        Square(self.data.from_sq())
    }

    fn to_sq(&self) -> Square {
        Square(self.data.to_sq())
    }

    pub fn is_ep_capture(&self) -> bool {
        self.data.is_ep_capture() != 0
    }

    pub fn into_bytes(&self) -> [u8; 4] {
        self.data.into_bytes()
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self { data: PackedData::from_bytes(bytes) }
    }
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
    if from.piece_type() != PieceType::Pawn {
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

pub fn make_move(pos: &mut Position, m: &Move) -> Snapshot {
    // @TODO: refactor this code, pretty please

    let castling = pos.castling;
    let en_passant = pos.en_passant;
    let halfmove_clock = pos.halfmove_clock;
    let fullmove_number = pos.fullmove_number;

    let disabled_castling = drop_castling(
        pos,
        m.from_sq(),
        m.to_sq(),
        pos.get_piece(m.from_sq()),
        pos.get_piece(m.to_sq()),
    );

    let from = pos.get_piece(m.from_sq());
    do_move_ep(pos, m, from);

    debug_assert!(pos.occupancies[pos.side_to_move.as_usize()].test(m.from_sq().as_u8()));

    let from_sq = m.from_sq();
    let to_sq = m.to_sq();

    let to_piece = pos.get_piece(to_sq);

    move_piece(&mut pos.bitboards[from.as_usize()], from_sq, to_sq);

    if to_piece != Piece::NONE {
        pos.bitboards[to_piece.as_usize()].unset(m.to_sq().as_u8()); // Clear the 'to' square for the captured piece
    }

    do_castling(pos, m, from);
    post_move(pos);

    pos.castling &= !disabled_castling;
    pos.en_passant = update_en_passant_square(pos, m.from_sq(), m.to_sq(), from);

    Snapshot { castling, en_passant, halfmove_clock, fullmove_number, to_piece }
}

pub fn unmake_move(pos: &mut Position, m: &Move, snapshot: &Snapshot) {
    let from = pos.get_piece(m.to_sq());
    undo_move_generic(pos, m, from, snapshot.to_piece);
    undo_move_ep(pos, m, from);

    undo_castling(pos, m, from);
    post_move(pos);

    pos.restore(snapshot);
}

fn do_move_ep(pos: &mut Position, m: &Move, from: Piece) {
    let (to_file, _) = m.to_sq().file_rank();

    // check if it's an en passant capture
    if m.is_ep_capture() {
        // capture the opponent's pawn passed en passant square
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(from.color().opponent(), PieceType::Pawn);

        debug_assert!(pos.get_piece(enemy_sq) == enemy);
        debug_assert!(pos.get_piece(m.to_sq()) == Piece::NONE);

        // Remove the captured pawn from the board
        pos.bitboards[enemy.as_usize()].unset(enemy_sq.as_u8());
    }
}

fn undo_move_ep(pos: &mut Position, m: &Move, from: Piece) {
    if m.is_ep_capture() {
        // Restore the captured pawn on the en passant square
        let (to_file, _) = m.to_sq().file_rank();
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(from.color().opponent(), PieceType::Pawn);

        debug_assert!(pos.get_piece(enemy_sq) == Piece::NONE);

        // Place the captured pawn back on the board
        pos.bitboards[enemy.as_usize()].set(enemy_sq.as_u8());
    }
}

fn move_piece(board: &mut BitBoard, from_sq: Square, to_sq: Square) {
    debug_assert!(board.test(from_sq.as_u8()), "No piece found on 'from' square");
    board.unset(from_sq.as_u8());
    board.set(to_sq.as_u8());
}

fn undo_move_generic(pos: &mut Position, m: &Move, from: Piece, to: Piece) {
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

fn do_castling(pos: &mut Position, m: &Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }
    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], from_sq, to_sq);
}

fn undo_castling(pos: &mut Position, m: &Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }

    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], to_sq, from_sq);
}

fn post_move(pos: &mut Position) {
    pos.change_side();
    pos.update_cache();
}

#[cfg(test)]
mod tests {}
