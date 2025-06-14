use paste::paste;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::engine::types::Piece;

pub mod constants {
    pub const RANK_1: u8 = 0;
    pub const RANK_2: u8 = 1;
    pub const RANK_3: u8 = 2;
    pub const RANK_4: u8 = 3;
    pub const RANK_5: u8 = 4;
    pub const RANK_6: u8 = 5;
    pub const RANK_7: u8 = 6;
    pub const RANK_8: u8 = 7;

    pub const FILE_A: u8 = 0;
    pub const FILE_B: u8 = 1;
    pub const FILE_C: u8 = 2;
    pub const FILE_D: u8 = 3;
    pub const FILE_E: u8 = 4;
    pub const FILE_F: u8 = 5;
    pub const FILE_G: u8 = 6;
    pub const FILE_H: u8 = 7;
}

/* #region BitBoard */

/// A `BitBoard` is a 64-bit representation of a chessboard, where each bit
/// corresponds to a square on the board.
///
/// Typically, bit 0 represents square A1 and bit 63 represents H8, following
/// little-endian rank-file mapping (LSB = A1, MSB = H8).
///
/// Bitboards are commonly used for fast move generation and board representation
/// in chess engines, allowing bitwise operations to perform bulk computations.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BitBoard(u64);

pub type Board = [BitBoard; Piece::COUNT];

impl BitBoard {
    pub const fn new() -> Self {
        Self(0u64)
    }

    pub const fn from(val: u64) -> Self {
        Self(val)
    }

    pub const fn from_bit(bit: u8) -> Self {
        Self(1u64 << bit)
    }

    pub const fn none(&self) -> bool {
        self.0 == 0
    }

    pub const fn any(&self) -> bool {
        self.0 != 0
    }

    pub const fn get(&self) -> u64 {
        self.0
    }

    pub const fn test(&self, bit: u8) -> bool {
        (self.0 & (1u64 << bit)) != 0
    }

    pub const fn set(&mut self, bit: u8) {
        self.0 |= 1u64 << bit;
    }

    pub fn set_sq(&mut self, sq: Square) {
        self.set(sq.0);
    }

    pub const fn unset(&mut self, bit: u8) {
        self.0 &= !(1u64 << bit);
    }

    pub const fn equal(&self, val: u64) -> bool {
        self.0 == val
    }

    pub fn shift(&self, dir: i32) -> BitBoard {
        let val = if dir < 0 { self.0 >> (-dir) } else { self.0 << dir };
        BitBoard(val)
    }
}

/* #region Bitwise Operations */
impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> BitBoard {
        BitBoard::from(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> BitBoard {
        BitBoard::from(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> BitBoard {
        BitBoard::from(!self.0)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for rank in 0..8 {
            for file in 0..8 {
                let sq: u8 = rank * 8 + file;
                if self.test(sq) {
                    write!(f, "1 ")?;
                } else {
                    write!(f, "0 ")?;
                }
            }
            write!(f, "\n")?
        }
        Ok(())
    }
}

/* #endregion BitBoard */

/* #region Square */
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square(pub u8);

macro_rules! square_consts {
    ($($rank:literal),*) => {
        paste! {
            $(
                pub const [<A $rank>]: Square = Square(8 * ($rank - 1) + 0);
                pub const [<B $rank>]: Square = Square(8 * ($rank - 1) + 1);
                pub const [<C $rank>]: Square = Square(8 * ($rank - 1) + 2);
                pub const [<D $rank>]: Square = Square(8 * ($rank - 1) + 3);
                pub const [<E $rank>]: Square = Square(8 * ($rank - 1) + 4);
                pub const [<F $rank>]: Square = Square(8 * ($rank - 1) + 5);
                pub const [<G $rank>]: Square = Square(8 * ($rank - 1) + 6);
                pub const [<H $rank>]: Square = Square(8 * ($rank - 1) + 7);
            )*
        }
    };
}

impl Square {
    square_consts!(1, 2, 3, 4, 5, 6, 7, 8);

    pub const fn make(file: u8, rank: u8) -> Square {
        debug_assert!(file < 8 && rank < 8);
        Square((rank << 3) + file)
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn as_u16(&self) -> u16 {
        self.0 as u16
    }

    pub const fn file_rank(&self) -> (u8, u8) {
        debug_assert!(self.0 < 64);
        let file = self.0 & 0b111;
        let rank = self.0 >> 3;
        (file, rank)
    }

    pub const fn to_bitboard(&self) -> BitBoard {
        BitBoard::from_bit(self.0)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (file, rank) = self.file_rank();
        write!(f, "{}{}", (b'a' + file) as char, rank + 1)
    }
}
/* #endregion Square */

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
    use super::constants::*;
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
