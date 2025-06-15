/// A `BitBoard` is a 64-bit representation of a chessboard, where each bit
/// corresponds to a square on the board.
///
/// Typically, bit 0 represents square A1 and bit 63 represents H8, following
/// little-endian rank-file mapping (LSB = A1, MSB = H8).
///
/// Bitboards are commonly used for fast move generation and board representation
/// in chess engines, allowing bitwise operations to perform bulk computations.
///
use super::super::types::Piece;
use super::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

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

    pub fn first_nonzero_sq(&self) -> Square {
        Square(self.0.trailing_zeros() as u8)
    }

    /// clear the least-significant bit from a bitboard
    ///           A = 0b1001001000
    ///       A - 1 = 0b1001000111
    /// A & (A - 1) = 0b1001000000
    pub fn remove_first_nonzero_sq(&mut self) {
        self.0 &= self.0 - 1; // Clear the least significant bit
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq: u8 = rank * 8 + file;
                s.push(if self.test(sq) { '1' } else { '0' });
                s.push(' ');
            }
            s.push('\n');
        }
        s
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
