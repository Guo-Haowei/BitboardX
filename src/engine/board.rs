use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::engine::piece::Piece;

// Constants for ranks
pub const RANK_1: u8 = 0;
pub const RANK_2: u8 = 1;
pub const RANK_3: u8 = 2;
pub const RANK_4: u8 = 3;
pub const RANK_5: u8 = 4;
pub const RANK_6: u8 = 5;
pub const RANK_7: u8 = 6;
pub const RANK_8: u8 = 7;

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

impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);

    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);

    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);

    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);

    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);

    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    pub const fn make(file: u8, rank: u8) -> Square {
        debug_assert!(file < 8 && rank < 8);
        Square((rank << 3) + file)
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
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
        write!(f, "(f: {}, r: {})", file, rank)
    }
}
/* #endregion Square */

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Square::A8.file_rank(), (0, RANK_8));
        assert_eq!(Square::B7.file_rank(), (1, RANK_7));
        assert_eq!(Square::C6.file_rank(), (2, RANK_6));
        assert_eq!(Square::D5.file_rank(), (3, RANK_5));
        assert_eq!(Square::E4.file_rank(), (4, RANK_4));
        assert_eq!(Square::F3.file_rank(), (5, RANK_3));
        assert_eq!(Square::G2.file_rank(), (6, RANK_2));
        assert_eq!(Square::H1.file_rank(), (7, RANK_1));
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
