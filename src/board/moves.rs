use super::bitboard::BitBoard;
use super::piece::*;
use super::position::Position;
use super::types::*;
use crate::board::move_generator;

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

// if castling rights are already disabled, return
// if king moved, disable castling rights, return
// if rook moved, disable castling rights, return
// if rook taken out, disable castling rights, return
fn move_disable_castling<const BIT: u8>(
    pos: &Position,
    from: Piece,
    to: Piece,
    from_sq: Square,
    to_sq: Square,
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

// @TODO: make this function a psuedo-legal move generator
pub fn create_move(pos: &Position, from_sq: Square, to_sq: Square) -> Option<Move> {
    if !pos.occupancies[pos.side_to_move.as_usize()].test(from_sq.as_u8()) {
        return None;
    }

    let mut from = Piece::NONE;
    let mut to = Piece::NONE;
    for i in 0..pos.bitboards.len() {
        let bb = &pos.bitboards[i];
        if bb.test(from_sq.as_u8()) {
            from = unsafe { std::mem::transmute(i as u8) };
        }
        if bb.test(to_sq.as_u8()) {
            to = unsafe { std::mem::transmute(i as u8) };
        }
    }

    debug_assert!(from != Piece::NONE, "No piece found on 'from' square");
    let mut flags = 0u8;
    flags |= move_disable_castling::<{ MoveFlags::K }>(pos, from, to, from_sq, to_sq);
    flags |= move_disable_castling::<{ MoveFlags::Q }>(pos, from, to, from_sq, to_sq);
    flags |= move_disable_castling::<{ MoveFlags::k }>(pos, from, to, from_sq, to_sq);
    flags |= move_disable_castling::<{ MoveFlags::q }>(pos, from, to, from_sq, to_sq);

    Some(Move::new(from_sq, to_sq, from, to, flags))
}

pub fn validate_move(pos: &mut Position, m: &Move) -> bool {
    let us = pos.side_to_move;
    let opponent = us.opponent();
    let piece: Piece = Piece::get_piece(us, PieceType::King);
    debug_assert!(piece == Piece::W_KING || piece == Piece::B_KING);
    debug_assert!(piece.color() == us);

    do_move(pos, m);

    let legal = (pos.bitboards[piece.as_usize()] & pos.attack_map[opponent.as_usize()]).none();

    undo_move(pos, m);

    legal
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

pub fn create_move_verified(pos: &mut Position, from_sq: Square, to_sq: Square) -> Option<Move> {
    if !move_generator::pseudo_legal_move(pos, from_sq).test(to_sq.as_u8()) {
        return None;
    }

    create_move(pos, from_sq, to_sq)
}

pub fn parse_move(input: &str) -> Option<(Square, Square)> {
    if input.len() != 4 {
        return None;
    }

    let from_file = input.chars().nth(0)? as u8 - b'a';
    let from_rank = input.chars().nth(1)? as u8 - b'1';
    let to_file = input.chars().nth(2)? as u8 - b'a';
    let to_rank = input.chars().nth(3)? as u8 - b'1';

    if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
        return None;
    }

    Some((Square::make(from_file, from_rank), Square::make(to_file, to_rank)))
}

pub fn apply_move_str(pos: &mut Position, move_str: &str) -> bool {
    match parse_move(move_str) {
        None => false,
        Some((from, to)) => match create_move_verified(pos, from, to) {
            None => false,
            Some(m) => {
                do_move(pos, &m);
                true
            }
        },
    }
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

    #[test]
    fn test_parse_move() {
        assert_eq!(parse_move("e2e4"), Some((Square::E2, Square::E4)));
        assert_eq!(parse_move("a7a8"), Some((Square::A7, Square::A8)));
        assert_eq!(parse_move("h1h2"), Some((Square::H1, Square::H2)));
        assert_eq!(parse_move("d4d5"), Some((Square::D4, Square::D5)));
        assert_eq!(parse_move("z1z2"), None);
        assert_eq!(parse_move("e9e4"), None);
        assert_eq!(parse_move("e2e"), None);
    }

    #[test]
    fn test_move_validation() {
        // 2 . . . . . . . k
        // 1 K B . . . . . r
        //   a b c d e f g h
        let mut pos = Position::from("8/8/8/8/8/8/7k/KB5r w - - 0 1").unwrap();

        assert_eq!(
            pos.attack_map[Color::BLACK.as_usize()],
            BitBoard::from(0b11000000_01000000_01111110)
        );

        let m = create_move(&pos, Square::B1, Square::A2).unwrap();

        assert!(!validate_move(&mut pos, &m), "Move bishop to A2 exposes king to check");
    }
}
