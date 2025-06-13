use super::board::{BitBoard, Square};
use super::piece::*;
use super::position::Position;

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

pub struct Move {
    pub from_sq: Square,
    pub to_sq: Square,
    pub pieces: u8, // encode from piece and to piece,
    pub flags: u8,  // reserved for castling, en passant, promotion
}

// A move needs 16 bits to be stored
//
// bit  0- 5: destination square (from 0 to 63)
// bit  6-11: origin square (from 0 to 63)
// bit 12-13: promotion piece type - 2 (from KNIGHT-2 to QUEEN-2)
// bit 14-15: special move flag: promotion (1), en passant (2), castling (3)
// NOTE: en passant bit is set only when a pawn can be captured
//
// Special cases are Move::none() and Move::null(). We can sneak these in because
// in any normal move the destination square and origin square are always different,
// but Move::none() and Move::null() have the same origin and destination square.

// class Move {
//    public:
//     Move() = default;
//     constexpr explicit Move(std::uint16_t d) :
//         data(d) {}

//     constexpr Move(Square from, Square to) :
//         data((from << 6) + to) {}

//     template<MoveType T>
//     static constexpr Move make(Square from, Square to, PieceType pt = KNIGHT) {
//         return Move(T + ((pt - KNIGHT) << 12) + (from << 6) + to);
//     }

//     constexpr Square from_sq() const {
//         assert(is_ok());
//         return Square((data >> 6) & 0x3F);
//     }

//     constexpr Square to_sq() const {
//         assert(is_ok());
//         return Square(data & 0x3F);
//     }

//     constexpr int from_to() const { return data & 0xFFF; }

//     constexpr MoveType type_of() const { return MoveType(data & (3 << 14)); }

//     constexpr PieceType promotion_type() const { return PieceType(((data >> 12) & 3) + KNIGHT); }

//     constexpr bool is_ok() const { return none().data != data && null().data != data; }

//     static constexpr Move null() { return Move(65); }
//     static constexpr Move none() { return Move(0); }

//     constexpr bool operator==(const Move& m) const { return data == m.data; }
//     constexpr bool operator!=(const Move& m) const { return data != m.data; }

//     constexpr explicit operator bool() const { return data != 0; }

//     constexpr std::uint16_t raw() const { return data; }

//     struct MoveHash {
//         std::size_t operator()(const Move& m) const { return make_key(m.data); }
//     };

//    protected:
//     std::uint16_t data;
// };

impl Move {
    const PIECE_MASK: u8 = 0xF;
    const CAPTURE_MASK: u8 = 0xF0;

    pub fn new(from_sq: Square, to_sq: Square, piece: Piece, capture: Piece, flags: u8) -> Self {
        debug_assert!(piece != Piece::NONE);

        let pieces =
            (piece.as_u8()) & Self::PIECE_MASK | ((capture.as_u8()) << 4) & Self::CAPTURE_MASK;
        Self { from_sq, to_sq, pieces, flags }
    }

    pub fn piece(&self) -> Piece {
        let bits = self.pieces & 0b1111;
        let piece = unsafe { std::mem::transmute(bits) };
        piece
    }

    pub fn capture(&self) -> Piece {
        let bits = (self.pieces & Self::CAPTURE_MASK) >> 4;
        unsafe { std::mem::transmute(bits) }
    }

    pub fn castling_mask(&self) -> u8 {
        self.flags & MoveFlags::KQkq
    }

    pub fn castling_type(&self) -> Castling {
        match self.piece() {
            Piece::W_KING if self.to_sq == Square::G1 => Castling::WhiteKingSide,
            Piece::W_KING if self.to_sq == Square::C1 => Castling::WhiteQueenSide,
            Piece::B_KING if self.to_sq == Square::G8 => Castling::BlackKingSide,
            Piece::B_KING if self.to_sq == Square::C8 => Castling::BlackQueenSide,
            _ => Castling::None,
        }
    }
}

fn do_move_generic(pos: &mut Position, m: &Move) {
    debug_assert!(pos.occupancies[pos.side_to_move.as_usize()].test(m.from_sq.as_u8()));

    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.bitboards[from.as_usize()];

    // @TODO: make set unset a generic function
    bb_from.unset(m.from_sq.as_u8()); // Clear the 'from' square
    bb_from.set(m.to_sq.as_u8()); // Place piece on 'to' square
    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].unset(m.to_sq.as_u8()); // Clear the 'to' square for the captured piece
    }
}

fn undo_move_generic(pos: &mut Position, m: &Move) {
    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.bitboards[from.as_usize()];
    bb_from.set(m.from_sq.as_u8()); // Place piece back on 'from' square
    bb_from.unset(m.to_sq.as_u8()); // Clear the 'to' square

    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].set(m.to_sq.as_u8()); // Place captured piece back on 'to' square
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
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let m = Move::new(Square::E7, Square::E8, Piece::W_QUEEN, Piece::B_KNIGHT, 0);
        assert_eq!(m.piece(), Piece::W_QUEEN);
        assert_eq!(m.capture(), Piece::B_KNIGHT);

        let m = Move::new(Square::E7, Square::E8, Piece::B_QUEEN, Piece::NONE, 0);
        assert_eq!(m.piece(), Piece::B_QUEEN);
        assert_eq!(m.capture(), Piece::NONE);
    }
}
