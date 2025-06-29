/// A `BitBoard` is a 64-bit representation of a chessboard, where each bit
/// corresponds to a square on the board.
///
/// Typically, bit 0 represents square A1 and bit 63 represents H8, following
/// little-endian rank-file mapping (LSB = A1, MSB = H8).
///
/// Bitboards are commonly used for fast move generation and board representation
/// in chess engines, allowing bitwise operations to perform bulk computations.
///
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::core::types::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BitBoard(u64);

pub type Board = [BitBoard; Piece::COUNT];

impl BitBoard {
    pub const N: i32 = 8;
    pub const S: i32 = -BitBoard::N;
    pub const E: i32 = 1;
    pub const W: i32 = -BitBoard::E;
    pub const NE: i32 = BitBoard::N + BitBoard::E;
    pub const NW: i32 = BitBoard::N + BitBoard::W;
    pub const SE: i32 = BitBoard::S + BitBoard::E;
    pub const SW: i32 = BitBoard::S + BitBoard::W;

    pub const MASK_A: u64 = 0x0101010101010101;
    pub const MASK_B: u64 = 0x0202020202020202;
    pub const MASK_C: u64 = 0x0404040404040404;
    pub const MASK_D: u64 = 0x0808080808080808;
    pub const MASK_E: u64 = 0x1010101010101010;
    pub const MASK_F: u64 = 0x2020202020202020;
    pub const MASK_G: u64 = 0x4040404040404040;
    pub const MASK_H: u64 = 0x8080808080808080;

    pub const MASK_1: u64 = 0x00000000000000FF;
    pub const MASK_2: u64 = 0x000000000000FF00;
    pub const MASK_3: u64 = 0x0000000000FF0000;
    pub const MASK_4: u64 = 0x00000000FF000000;
    pub const MASK_5: u64 = 0x000000FF00000000;
    pub const MASK_6: u64 = 0x0000FF0000000000;
    pub const MASK_7: u64 = 0x00FF000000000000;
    pub const MASK_8: u64 = 0xFF00000000000000;

    #[inline(always)]
    pub const fn new() -> Self {
        Self(0u64)
    }

    #[inline(always)]
    pub const fn from(val: u64) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub const fn none(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn any(&self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub const fn get(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn test(&self, bit: u8) -> bool {
        (self.0 & (1u64 << bit)) != 0
    }

    #[inline(always)]
    pub const fn set(&mut self, bit: u8) {
        self.0 |= 1u64 << bit;
    }

    #[inline(always)]
    pub const fn test_sq(&self, sq: Square) -> bool {
        debug_assert!(sq.as_u8() < 64, "Square out of bounds");
        self.test(sq.as_u8())
    }

    #[inline(always)]
    pub const fn set_sq(&mut self, sq: Square) {
        debug_assert!(sq.as_u8() < 64, "Square out of bounds");
        self.set(sq.as_u8());
    }

    #[inline(always)]
    pub const fn unset(&mut self, bit: u8) {
        self.0 &= !(1u64 << bit);
    }

    #[inline(always)]
    pub const fn equal(&self, val: u64) -> bool {
        self.0 == val
    }

    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq: u8 = rank * 8 + file;
                s.push(if self.test(sq) { 'X' } else { '.' });
                if file < 7 {
                    s.push(' ');
                }
            }
            s.push('\n');
        }
        s
    }

    pub fn iter(&self) -> BitBoardIter {
        BitBoardIter { remaining: self.0 }
    }
}

pub struct BitBoardIter {
    remaining: u64,
}

impl Iterator for BitBoardIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            let tz = self.remaining.trailing_zeros();
            // clear the least-significant bit from a bitboard
            //           A = 0b1001001000
            //       A - 1 = 0b1001000111
            // A & (A - 1) = 0b1001000000
            self.remaining &= self.remaining - 1;
            Some(Square::new(tz as u8))
        }
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> BitBoard {
        BitBoard::from(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> BitBoard {
        BitBoard::from(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn not(self) -> BitBoard {
        BitBoard::from(!self.0)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_iterator() {
        let bb = BitBoard::from(0b1110000111100010);

        let mut idx = 0;
        let squares = [1, 5, 6, 7, 8, 13, 14, 15];

        for sq in bb.iter() {
            assert!(bb.test_sq(sq));
            assert_eq!(sq.as_u8(), squares[idx], "Square mismatch at index {}", idx);
            idx += 1;
        }
    }
}
