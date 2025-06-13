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
    pub from_sq: u8,
    pub to_sq: u8,
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

    pub fn new(from_sq: u8, to_sq: u8, piece: Piece, capture: Piece, flags: u8) -> Self {
        debug_assert!(from_sq < 64 && to_sq < 64);
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
            Piece::W_KING if self.to_sq == SQ_G1 => Castling::WhiteKingSide,
            Piece::W_KING if self.to_sq == SQ_C1 => Castling::WhiteQueenSide,
            Piece::B_KING if self.to_sq == SQ_G8 => Castling::BlackKingSide,
            Piece::B_KING if self.to_sq == SQ_C8 => Castling::BlackQueenSide,
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
    from_sq: u8,
    to_sq: u8,
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
        (Piece::W_ROOK, SQ_A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
        (Piece::W_ROOK, SQ_H1) if (BIT & MoveFlags::K) != 0 => return BIT,
        (Piece::B_ROOK, SQ_A8) if (BIT & MoveFlags::q) != 0 => return BIT,
        (Piece::B_ROOK, SQ_H8) if (BIT & MoveFlags::k) != 0 => return BIT,
        _ => {}
    }

    match (to, to_sq) {
        (Piece::W_ROOK, SQ_A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
        (Piece::W_ROOK, SQ_H1) if (BIT & MoveFlags::K) != 0 => return BIT,
        (Piece::B_ROOK, SQ_A8) if (BIT & MoveFlags::q) != 0 => return BIT,
        (Piece::B_ROOK, SQ_H8) if (BIT & MoveFlags::k) != 0 => return BIT,
        _ => {}
    }

    0
}

// @TODO: make this function a psuedo-legal move generator
pub fn create_move(pos: &Position, from_sq: u8, to_sq: u8) -> Option<Move> {
    if !pos.occupancies[pos.side_to_move as usize].test(from_sq) {
        return None;
    }

    let mut from = Piece::NONE;
    let mut to = Piece::NONE;
    for i in 0..pos.bitboards.len() {
        let bb = &pos.bitboards[i];
        if bb.test(from_sq) {
            from = unsafe { std::mem::transmute(i as u8) };
        }
        if bb.test(to_sq) {
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
    let our_side = pos.side_to_move;
    let their_side = get_opposite_color(our_side);
    let piece: Piece = unsafe { std::mem::transmute(our_side * 6 + Piece::W_KING.as_u8()) };
    debug_assert!(piece == Piece::W_KING || piece == Piece::B_KING);
    debug_assert!(piece.color() == our_side);

    do_move(pos, m);

    let legal = (pos.bitboards[piece.as_usize()] & pos.attack_map[their_side as usize]).none();

    undo_move(pos, m);

    legal
}

fn do_move_generic(pos: &mut Position, m: &Move) {
    debug_assert!(pos.occupancies[pos.side_to_move as usize].test(m.from_sq));

    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.bitboards[from.as_usize()];

    bb_from.unset(m.from_sq); // Clear the 'from' square
    bb_from.set(m.to_sq); // Place piece on 'to' square
    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].unset(m.to_sq); // Clear the 'to' square for the captured piece
    }
}

fn undo_move_generic(pos: &mut Position, m: &Move) {
    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.bitboards[from.as_usize()];
    bb_from.set(m.from_sq); // Place piece back on 'from' square
    bb_from.unset(m.to_sq); // Clear the 'to' square

    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].set(m.to_sq); // Place captured piece back on 'to' square
    }
}

fn do_castling(pos: &mut Position, m: &Move) {
    // Update castling rights if necessary
    pos.castling &= !m.castling_mask();

    // Move rook to its new position
    match m.castling_type() {
        Castling::WhiteKingSide => {
            pos.bitboards[Piece::W_ROOK.as_usize()].unset(SQ_H1);
            pos.bitboards[Piece::W_ROOK.as_usize()].set(SQ_F1);
        }
        Castling::WhiteQueenSide => {
            pos.bitboards[Piece::W_ROOK.as_usize()].unset(SQ_A1);
            pos.bitboards[Piece::W_ROOK.as_usize()].set(SQ_D1);
        }
        Castling::BlackKingSide => {
            pos.bitboards[Piece::B_ROOK.as_usize()].unset(SQ_H8);
            pos.bitboards[Piece::B_ROOK.as_usize()].set(SQ_F8);
        }
        Castling::BlackQueenSide => {
            pos.bitboards[Piece::B_ROOK.as_usize()].unset(SQ_A8);
            pos.bitboards[Piece::B_ROOK.as_usize()].set(SQ_D8);
        }
        Castling::None => {}
    }
}

fn undo_castling(pos: &mut Position, m: &Move) {
    // Restore castling rights if necessary
    pos.castling |= m.castling_mask();

    // Move rook back to its original position
    match m.castling_type() {
        // @TODO: extract position.move_piece()
        Castling::WhiteKingSide => {
            pos.bitboards[Piece::W_ROOK.as_usize()].unset(SQ_F1);
            pos.bitboards[Piece::W_ROOK.as_usize()].set(SQ_H1);
        }
        Castling::WhiteQueenSide => {
            pos.bitboards[Piece::W_ROOK.as_usize()].unset(SQ_D1);
            pos.bitboards[Piece::W_ROOK.as_usize()].set(SQ_A1);
        }
        Castling::BlackKingSide => {
            pos.bitboards[Piece::B_ROOK.as_usize()].unset(SQ_F8);
            pos.bitboards[Piece::B_ROOK.as_usize()].set(SQ_H8);
        }
        Castling::BlackQueenSide => {
            pos.bitboards[Piece::B_ROOK.as_usize()].unset(SQ_D8);
            pos.bitboards[Piece::B_ROOK.as_usize()].set(SQ_A8);
        }
        Castling::None => {}
    }
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

pub fn create_move_verified(pos: &mut Position, from_sq: u8, to_sq: u8) -> Option<Move> {
    if (move_generator::pseudo_legal_move(pos, from_sq) & BitBoard::from_bit(to_sq)).none() {
        return None;
    }

    create_move(pos, from_sq, to_sq)
}

pub fn parse_move(input: &str) -> Option<(u8, u8)> {
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

    Some((make_square(from_file, from_rank), make_square(to_file, to_rank)))
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
        let m = Move::new(SQ_E7, SQ_E8, Piece::W_QUEEN, Piece::B_KNIGHT, 0);
        assert_eq!(m.piece(), Piece::W_QUEEN);
        assert_eq!(m.capture(), Piece::B_KNIGHT);

        let m = Move::new(SQ_E7, SQ_E8, Piece::B_QUEEN, Piece::NONE, 0);
        assert_eq!(m.piece(), Piece::B_QUEEN);
        assert_eq!(m.capture(), Piece::NONE);
    }

    #[test]
    fn test_parse_move() {
        assert_eq!(parse_move("e2e4"), Some((SQ_E2, SQ_E4)));
        assert_eq!(parse_move("a7a8"), Some((SQ_A7, SQ_A8)));
        assert_eq!(parse_move("h1h2"), Some((SQ_H1, SQ_H2)));
        assert_eq!(parse_move("d4d5"), Some((SQ_D4, SQ_D5)));
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
            pos.attack_map[COLOR_BLACK as usize],
            BitBoard::from(0b11000000_01000000_01111110)
        );

        let m = create_move(&pos, SQ_B1, SQ_A2).unwrap();

        assert!(!validate_move(&mut pos, &m), "Move bishop to A2 exposes king to check");
    }
}
