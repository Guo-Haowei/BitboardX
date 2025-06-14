use super::square::*;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveType {
    Normal = 0,
    Castling = 1,
    EnPassant = 2,
    Promotion = 3,
}

/// Bit layout for a `Move` (16-bit packed):
///
/// ```text
/// 15  14  13  12   11        6   5        0
/// +---+---+---+---+----------+------------+
/// | P | P | F | F |  To[5:0] | From[5:0]  |
/// +---+---+---+---+----------+------------+
///  2 bits  2 bits    6 bits     6 bits
///  [14-15] [12-13]   [6–11]     [0–5]
/// ```
///
/// - `from` (0–5): source square (0–63)
/// - `to` (6–11): destination square (0–63)
/// - `flag` (12–13): move type (e.g., castle, en passant, promotion)
/// - `promo` (14–15): promotion piece (0 = knight, 1 = bishop, 2 = rook, 3 = queen)

#[derive(Debug, Clone, Copy)]
pub struct Move(u16);

impl Move {
    const SQUARE_MASK: u16 = 0b111111; // 6 bits for square (0-63)

    pub fn new(from_sq: Square, to_sq: Square, move_type: MoveType) -> Self {
        let mut data = 0u16;
        data |= from_sq.as_u16();
        data |= to_sq.as_u16() << 6;
        data |= (move_type as u16) << 12;

        Self(data)
    }

    pub fn from_sq(&self) -> Square {
        Square((self.0 & Self::SQUARE_MASK) as u8)
    }

    pub fn to_sq(&self) -> Square {
        Square(((self.0 >> 6) & Self::SQUARE_MASK) as u8)
    }

    pub fn get_type(&self) -> MoveType {
        let bits = (self.0 >> 12) & 0b11;
        unsafe { std::mem::transmute::<u8, MoveType>(bits as u8) }
    }

    // @TODO: promotion piece
}

#[cfg(test)]
mod tests {
    use super::super::bitboard::BitBoard;
    use super::super::constants::*;
    use super::*;

    #[test]
    fn test_move_creation() {
        let m = Move::new(Square::E2, Square::E4, MoveType::Castling);
        assert_eq!(m.from_sq(), Square::E2);
        assert_eq!(m.to_sq(), Square::E4);
        assert_eq!(m.get_type(), MoveType::Castling);
    }

    #[test]
    fn make_square_test() {
        assert_eq!(Square::make(0, RANK_8), Square::A8);
        assert_eq!(Square::make(1, RANK_7), Square::B7);
        assert_eq!(Square::make(2, RANK_6), Square::C6);
        assert_eq!(Square::make(3, RANK_5), Square::D5);
        assert_eq!(Square::make(4, RANK_4), Square::E4);
        assert_eq!(Square::make(5, RANK_3), Square::F3);
        assert_eq!(Square::make(6, RANK_2), Square::G2);
        assert_eq!(Square::make(7, RANK_1), Square::H1);
    }

    #[test]
    fn get_file_rank_test() {
        assert_eq!(Square::A8.file_rank(), (FILE_A, RANK_8));
        assert_eq!(Square::B7.file_rank(), (FILE_B, RANK_7));
        assert_eq!(Square::C6.file_rank(), (FILE_C, RANK_6));
        assert_eq!(Square::D5.file_rank(), (FILE_D, RANK_5));
        assert_eq!(Square::E4.file_rank(), (FILE_E, RANK_4));
        assert_eq!(Square::F3.file_rank(), (FILE_F, RANK_3));
        assert_eq!(Square::G2.file_rank(), (FILE_G, RANK_2));
        assert_eq!(Square::H1.file_rank(), (FILE_H, RANK_1));
    }

    #[test]
    fn test_bitboard() {
        let mut bb = BitBoard::new();
        assert!(!bb.test(0));
        assert!(!bb.test(63));

        bb.set(0);
        assert!(bb.test(0));
        assert!(!bb.test(1));

        bb.set(63);
        assert!(bb.test(63));
        assert!(!bb.test(62));

        let bb2 = BitBoard::from(1u64 << 1);
        let bb3 = BitBoard::from(1u64 << 2);
        let bb4 = bb2 | bb3;
        assert!(bb4.test(1));
        assert!(bb4.test(2));
    }
}
