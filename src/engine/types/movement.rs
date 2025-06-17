use crate::engine::types::PieceType;

use super::square::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CastlingType {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
    None,
}

pub struct CastlingRight;

#[allow(non_upper_case_globals)]
impl CastlingRight {
    pub const K: u8 = 1u8 << CastlingType::WhiteKingSide as u8;
    pub const Q: u8 = 1u8 << CastlingType::WhiteQueenSide as u8;
    pub const k: u8 = 1u8 << CastlingType::BlackKingSide as u8;
    pub const q: u8 = 1u8 << CastlingType::BlackQueenSide as u8;
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

    pub fn none() -> Self {
        Self(0)
    }

    pub fn new(
        src_sq: Square,
        dst_sq: Square,
        move_type: MoveType,
        promotion: Option<PieceType>,
    ) -> Self {
        let mut data = 0u16;
        data |= src_sq.as_u16();
        data |= dst_sq.as_u16() << 6;
        data |= (move_type as u16) << 12;

        if let Some(promo) = promotion {
            debug_assert!(move_type == MoveType::Promotion);
            debug_assert!(matches!(
                promo,
                PieceType::Knight | PieceType::Bishop | PieceType::Rook | PieceType::Queen
            ));

            data |= (promo as u16 - 1) << 14; // Promotion piece
        } else {
            debug_assert!(move_type != MoveType::Promotion);
        }

        Self(data)
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub fn src_sq(&self) -> Square {
        Square::from_u8((self.0 & Self::SQUARE_MASK) as u8)
    }

    pub fn dst_sq(&self) -> Square {
        Square::from_u8(((self.0 >> 6) & Self::SQUARE_MASK) as u8)
    }

    pub fn get_type(&self) -> MoveType {
        let bits = (self.0 >> 12) & 0b11;
        unsafe { std::mem::transmute::<u8, MoveType>(bits as u8) }
    }

    pub fn get_promotion(&self) -> Option<PieceType> {
        if self.get_type() == MoveType::Promotion {
            let promo_bits = ((self.0 >> 14) & 0b11) + 1;
            match promo_bits {
                1..=4 => Some(unsafe { std::mem::transmute::<u8, PieceType>(promo_bits as u8) }),
                _ => panic!("Invalid promotion bits: {}", promo_bits), // Should never happen
            }
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        let from = self.src_sq();
        let to = self.dst_sq();
        let promo = match self.get_promotion() {
            Some(PieceType::Knight) => "n",
            Some(PieceType::Bishop) => "b",
            Some(PieceType::Rook) => "r",
            Some(PieceType::Queen) => "q",
            None => "",
            _ => panic!("Invalid promotion piece"),
        };
        format!("{}{}{}", from.to_string(), to.to_string(), promo)
    }

    pub fn get_en_passant_capture(&self) -> Square {
        debug_assert!(self.get_type() == MoveType::EnPassant);
        let (_, from_rank) = self.src_sq().file_rank();
        let (to_file, _) = self.dst_sq().file_rank();

        Square::make(to_file, from_rank)
    }
}

pub struct MoveList {
    moves: [Move; 256],
    count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self { moves: [Move::none(); 256], count: 0 }
    }

    pub fn add(&mut self, m: Move) {
        if self.count < self.moves.len() {
            self.moves[self.count] = m;
            self.count += 1;
        } else {
            panic!("MoveList is full, cannot add more moves");
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves.iter().take(self.count)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn get(&self, index: usize) -> Option<Move> {
        if index >= self.count {
            return None;
        }
        Some(self.moves[index])
    }
}

#[cfg(test)]
mod tests {
    use super::super::bitboard::BitBoard;
    use super::*;

    #[test]
    fn castling_move_creation() {
        let m = Move::new(Square::E2, Square::E4, MoveType::Castling, None);
        assert_eq!(m.src_sq(), Square::E2);
        assert_eq!(m.dst_sq(), Square::E4);
        assert_eq!(m.get_type(), MoveType::Castling);
        assert_eq!(m.get_promotion(), None);
    }

    #[test]
    fn promotion_move_creation() {
        let m = Move::new(Square::E7, Square::E8, MoveType::Promotion, Some(PieceType::Queen));
        assert_eq!(m.src_sq(), Square::E7);
        assert_eq!(m.dst_sq(), Square::E8);
        assert_eq!(m.get_type(), MoveType::Promotion);
        assert_eq!(m.get_promotion(), Some(PieceType::Queen));

        let m = Move::new(Square::E7, Square::E8, MoveType::Promotion, Some(PieceType::Rook));
        assert_eq!(m.get_promotion(), Some(PieceType::Rook));

        let m = Move::new(Square::E7, Square::E8, MoveType::Promotion, Some(PieceType::Knight));
        assert_eq!(m.get_promotion(), Some(PieceType::Knight));

        let m = Move::new(Square::E7, Square::E8, MoveType::Promotion, Some(PieceType::Bishop));
        assert_eq!(m.get_promotion(), Some(PieceType::Bishop));
    }

    #[test]
    fn make_square_test() {
        assert_eq!(Square::make(File::A, Rank::_8), Square::A8);
        assert_eq!(Square::make(File::B, Rank::_7), Square::B7);
        assert_eq!(Square::make(File::C, Rank::_6), Square::C6);
        assert_eq!(Square::make(File::D, Rank::_5), Square::D5);
        assert_eq!(Square::make(File::E, Rank::_4), Square::E4);
        assert_eq!(Square::make(File::F, Rank::_3), Square::F3);
        assert_eq!(Square::make(File::G, Rank::_2), Square::G2);
        assert_eq!(Square::make(File::H, Rank::_1), Square::H1);
    }

    #[test]
    fn get_file_rank_test() {
        assert_eq!(Square::A8.file_rank(), (File::A, Rank::_8));
        assert_eq!(Square::B7.file_rank(), (File::B, Rank::_7));
        assert_eq!(Square::C6.file_rank(), (File::C, Rank::_6));
        assert_eq!(Square::D5.file_rank(), (File::D, Rank::_5));
        assert_eq!(Square::E4.file_rank(), (File::E, Rank::_4));
        assert_eq!(Square::F3.file_rank(), (File::F, Rank::_3));
        assert_eq!(Square::G2.file_rank(), (File::G, Rank::_2));
        assert_eq!(Square::H1.file_rank(), (File::H, Rank::_1));
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
