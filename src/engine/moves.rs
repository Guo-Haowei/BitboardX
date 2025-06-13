use super::board::{BitBoard, Square};
use super::piece::*;
use super::position::Position;
use modular_bitfield::prelude::*;

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
    color: B1,
    piece: B4,
    capture: B5,
    castling: B4,
    #[skip]
    __: B6,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    data: PackedData,
}

impl Move {
    pub fn new(from_sq: Square, to_sq: Square, piece: Piece, capture: Piece, casling: u8) -> Self {
        debug_assert!(piece != Piece::NONE);
        let mut data = PackedData::new();
        data.set_from_sq(from_sq.as_u8());
        data.set_to_sq(to_sq.as_u8());
        data.set_piece(piece.as_u8());
        data.set_capture(capture.as_u8());
        data.set_castling(casling);

        Self { data }
    }

    fn from_sq(&self) -> Square {
        Square(self.data.from_sq())
    }

    fn to_sq(&self) -> Square {
        Square(self.data.to_sq())
    }

    pub fn color(&self) -> Color {
        if self.data.color() == 0 { Color::WHITE } else { Color::BLACK }
    }

    pub fn piece(&self) -> Piece {
        Piece::from(self.data.piece())
    }

    pub fn capture(&self) -> Piece {
        Piece::from(self.data.capture())
    }

    pub fn castling_mask(&self) -> u8 {
        self.data.castling() & MoveFlags::KQkq
    }

    pub fn castling_type(&self) -> Castling {
        match self.piece() {
            Piece::W_KING if self.to_sq() == Square::G1 => Castling::WhiteKingSide,
            Piece::W_KING if self.to_sq() == Square::C1 => Castling::WhiteQueenSide,
            Piece::B_KING if self.to_sq() == Square::G8 => Castling::BlackKingSide,
            Piece::B_KING if self.to_sq() == Square::C8 => Castling::BlackQueenSide,
            _ => Castling::None,
        }
    }

    pub fn into_bytes(&self) -> [u8; 4] {
        self.data.into_bytes()
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self { data: PackedData::from_bytes(bytes) }
    }
}

fn do_move_generic(pos: &mut Position, m: &Move) {
    debug_assert!(pos.occupancies[pos.side_to_move.as_usize()].test(m.from_sq().as_u8()));

    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.bitboards[from.as_usize()];

    // @TODO: make set unset a generic function
    bb_from.unset(m.from_sq().as_u8()); // Clear the 'from' square
    bb_from.set(m.to_sq().as_u8()); // Place piece on 'to' square
    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].unset(m.to_sq().as_u8()); // Clear the 'to' square for the captured piece
    }
}

fn undo_move_generic(pos: &mut Position, m: &Move) {
    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.bitboards[from.as_usize()];
    bb_from.set(m.from_sq().as_u8()); // Place piece back on 'from' square
    bb_from.unset(m.to_sq().as_u8()); // Clear the 'to' square

    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].set(m.to_sq().as_u8()); // Place captured piece back on 'to' square
    }
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

fn do_castling(pos: &mut Position, m: &Move) {
    // Update castling rights if necessary
    pos.castling &= !m.castling_mask();

    let index = m.castling_type() as usize;
    if index >= CASTLING_ROOK_SQUARES.len() {
        return;
    }

    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index];
    move_piece(&mut pos.bitboards[piece.as_usize()], from_sq, to_sq);
}

fn undo_castling(pos: &mut Position, m: &Move) {
    // Restore castling rights if necessary
    pos.castling |= m.castling_mask();

    let index = m.castling_type() as usize;
    if index >= CASTLING_ROOK_SQUARES.len() {
        return;
    }

    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index];
    move_piece(&mut pos.bitboards[piece.as_usize()], to_sq, from_sq);
}

fn post_move(pos: &mut Position) {
    pos.change_side();
    pos.update_cache();
}

pub fn do_move(pos: &mut Position, m: &Move) -> bool {
    do_move_generic(pos, m);
    do_castling(pos, m);
    post_move(pos);
    true
}

pub fn undo_move(pos: &mut Position, m: &Move) {
    undo_move_generic(pos, m);
    undo_castling(pos, m);
    post_move(pos);
}

#[cfg(test)]
mod tests {}
