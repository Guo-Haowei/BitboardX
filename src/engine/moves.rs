use super::board::{BitBoard, RANK_3, RANK_4, RANK_5, RANK_6, Square};
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
    color: B1,
    capture: B5,
    castling_disabled: B4,
    drop_en_passant: B1,
    create_en_passant: B1,
    en_passant_sq: B4,
    #[skip]
    __: B4,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    data: PackedData,
}

impl Move {
    pub fn new(
        from_sq: Square,
        to_sq: Square,
        piece: Piece,
        capture: Piece,
        casling_disabled: u8,
        drop_en_passant: Option<Square>,
        create_en_passant: bool,
    ) -> Self {
        debug_assert!(piece != Piece::NONE);
        let color = piece.color();
        let mut data = PackedData::new();
        data.set_from_sq(from_sq.as_u8());
        data.set_to_sq(to_sq.as_u8());
        data.set_capture(capture.as_u8());
        data.set_castling_disabled(casling_disabled);
        data.set_color(color.as_u8());
        data.set_create_en_passant(create_en_passant as u8);

        match drop_en_passant {
            Some(sq) => {
                data.set_drop_en_passant(1);
                let (file, rank) = sq.file_rank();
                assert!(
                    matches!(rank, RANK_3 | RANK_6),
                    "En passant square must be on rank 3 or 6"
                );
                let value = if color == Color::WHITE { file } else { file + 8 };
                data.set_en_passant_sq(value);
            }
            None => {
                data.set_drop_en_passant(0);
            }
        }

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

    pub fn capture(&self) -> Piece {
        Piece::from(self.data.capture())
    }

    pub fn castling_mask(&self) -> u8 {
        self.data.castling_disabled() & MoveFlags::KQkq
    }

    pub fn drop_ep(&self) -> Option<Square> {
        if self.data.drop_en_passant() == 0 {
            return None;
        }
        let file = self.data.en_passant_sq() % 8;
        let rank = if file < 8 { RANK_3 } else { RANK_6 };
        Some(Square::make(file, rank))
    }

    pub fn create_ep(&self) -> bool {
        self.data.create_en_passant() != 0
    }

    pub fn into_bytes(&self) -> [u8; 4] {
        self.data.into_bytes()
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self { data: PackedData::from_bytes(bytes) }
    }
}

pub fn do_move(pos: &mut Position, m: &Move) -> bool {
    let from = pos.get_piece(m.from_sq());
    do_move_generic(pos, m, from);
    do_castling(pos, m, from);
    do_ep_move(pos, m, from);
    post_move(pos);
    true
}

pub fn undo_move(pos: &mut Position, m: &Move) {
    let from = pos.get_piece(m.to_sq());
    undo_move_generic(pos, m, from);
    undo_castling(pos, m, from);
    undo_ep_move(pos, m, from);
    post_move(pos);
}

fn do_ep_move(pos: &mut Position, m: &Move, from: Piece) {
    if !m.create_ep() {
        return;
    }

    debug_assert!(from.piece_type() == PieceType::Pawn, "En passant move must be a pawn move");
    let (file, rank) = m.to_sq().file_rank();
    match (rank, from.color()) {
        (RANK_4, Color::WHITE) => {
            pos.ep_sq = Some(Square::make(file, RANK_3));
        }
        (RANK_5, Color::BLACK) => {
            pos.ep_sq = Some(Square::make(file, RANK_6));
        }
        _ => {}
    }
}

fn undo_ep_move(pos: &mut Position, m: &Move, from: Piece) {
    // @TODO: Implement undo for en passant move
}

fn move_piece(board: &mut BitBoard, from_sq: Square, to_sq: Square) {
    debug_assert!(board.test(from_sq.as_u8()), "No piece found on 'from' square");
    board.unset(from_sq.as_u8());
    board.set(to_sq.as_u8());
}

fn do_move_generic(pos: &mut Position, m: &Move, from: Piece) {
    debug_assert!(pos.occupancies[pos.side_to_move.as_usize()].test(m.from_sq().as_u8()));

    let from_sq = m.from_sq();
    let to_sq = m.to_sq();
    let to = m.capture();

    move_piece(&mut pos.bitboards[from.as_usize()], from_sq, to_sq);

    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].unset(m.to_sq().as_u8()); // Clear the 'to' square for the captured piece
    }
}

fn undo_move_generic(pos: &mut Position, m: &Move, from: Piece) {
    let from_sq = m.from_sq();
    let to_sq = m.to_sq();
    let to = m.capture();

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
    // Update castling rights if necessary
    pos.castling &= !m.castling_mask();

    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }
    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], from_sq, to_sq);
}

fn undo_castling(pos: &mut Position, m: &Move, from: Piece) {
    // Restore castling rights if necessary
    pos.castling |= m.castling_mask();

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
