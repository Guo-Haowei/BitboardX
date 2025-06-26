use paste::paste;
use std::fmt;

use super::bitboard::BitBoard;
use crate::utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct File(pub u8);
impl File {
    pub const A: File = File(0);
    pub const B: File = File(1);
    pub const C: File = File(2);
    pub const D: File = File(3);
    pub const E: File = File(4);
    pub const F: File = File(5);
    pub const G: File = File(6);
    pub const H: File = File(7);

    pub fn diff(&self, other: File) -> i32 {
        (self.0 as i32) - (other.0 as i32)
    }

    pub fn west(&self) -> Option<File> {
        if self.0 == 0 { None } else { Some(File(self.0 - 1)) }
    }

    pub fn east(&self) -> Option<File> {
        if self.0 == 7 { None } else { Some(File(self.0 + 1)) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(pub u8);
impl Rank {
    pub const _1: Rank = Rank(0);
    pub const _2: Rank = Rank(1);
    pub const _3: Rank = Rank(2);
    pub const _4: Rank = Rank(3);
    pub const _5: Rank = Rank(4);
    pub const _6: Rank = Rank(5);
    pub const _7: Rank = Rank(6);
    pub const _8: Rank = Rank(7);

    pub fn diff(&self, other: Rank) -> i32 {
        (self.0 as i32) - (other.0 as i32)
    }

    pub fn north(&self) -> Option<Rank> {
        if self.0 == 7 { None } else { Some(Rank(self.0 + 1)) }
    }

    pub fn east(&self) -> Option<Rank> {
        if self.0 == 0 { None } else { Some(Rank(self.0 - 1)) }
    }
}

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square(u8);

impl Square {
    square_consts!(1, 2, 3, 4, 5, 6, 7, 8);

    pub const NONE: Square = Square(64);

    pub const fn make(file: File, rank: Rank) -> Square {
        Square(((rank.0 << 3) + file.0) as u8)
    }

    pub const fn new(value: u8) -> Square {
        debug_assert!(value < 64);
        Square(value)
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn as_u16(&self) -> u16 {
        self.0 as u16
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn file_rank(&self) -> (File, Rank) {
        debug_assert!(self.0 < 64);
        let f = self.0 & 0b111;
        let r = self.0 >> 3;
        (File(f), Rank(r))
    }

    pub const fn to_bitboard(&self) -> BitBoard {
        BitBoard::from(1u64 << self.0)
    }

    pub const fn is_none(&self) -> bool {
        self.0 >= 64
    }

    // Shoelace Formula (also called the Surveyor's Formula) for the area of a triangle in 2D space.
    // area = [ Ax * (By - Cy) + Bx * (Cy - Ay) + Cx * (Ay - By) ] / 2
    // but we only cares about the sign of the area, so we can skip the division by 2.
    pub fn same_line(&self, a: Square, b: Square) -> bool {
        let (ax, ay) = a.file_rank();
        let (bx, by) = b.file_rank();
        let (cx, cy) = self.file_rank();

        let two_signed_area =
            ax.0 as i32 * by.diff(cy) + bx.0 as i32 * cy.diff(ay) + cx.0 as i32 * ay.diff(by);

        two_signed_area == 0
    }

    // only returns true if square is between A and B
    pub fn same_line_inclusive(&self, a: Square, b: Square) -> bool {
        let (ax, ay) = a.file_rank();
        let (bx, by) = b.file_rank();
        let (cx, cy) = self.file_rank();

        if !self.same_line(a, b) {
            return false;
        }

        let (x_min, x_max) = utils::min_max(ax.0, bx.0);
        let (y_min, y_max) = utils::min_max(ay.0, by.0);
        let between_x = cx.0 >= x_min && cx.0 <= x_max;
        let between_y = cy.0 >= y_min && cy.0 <= y_max;

        between_x && between_y
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (file, rank) = self.file_rank();
        write!(f, "{}{}", (b'a' + file.0) as char, rank.0 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_line_diagonal() {
        let a = Square::A1;
        let b = Square::B2;
        let c = Square::C3;
        assert!(!a.same_line_inclusive(b, c));
        assert!(b.same_line_inclusive(a, c));
        assert!(!c.same_line_inclusive(a, b));

        let a = Square::A1;
        let b: Square = Square::B1;
        let c = Square::C3;
        assert!(!a.same_line_inclusive(b, c));
        assert!(!b.same_line_inclusive(a, c));
        assert!(!c.same_line_inclusive(a, b));
    }

    #[test]
    fn same_line_overlapping() {
        let a = Square::B2;
        let b = Square::B2;
        let c = Square::D8;
        assert!(!c.same_line_inclusive(a, b));
        assert!(a.same_line_inclusive(b, c));
        assert!(b.same_line_inclusive(a, c));
    }

    #[test]
    fn same_line_horizontal() {
        let a = Square::C1;
        let b = Square::C2;
        let c = Square::C5;
        assert!(!a.same_line_inclusive(b, c));
        assert!(b.same_line_inclusive(a, c));
        assert!(!c.same_line_inclusive(a, b));
    }

    #[test]
    fn same_line_vertical() {
        let a = Square::A1;
        let b = Square::A8;
        let c = Square::A3;
        assert!(!a.same_line_inclusive(b, c));
        assert!(!b.same_line_inclusive(a, c));
        assert!(c.same_line_inclusive(a, b));
    }

    #[test]
    fn more_same_line_test() {
        let a = Square::G8;
        let b = Square::B3;

        assert!(Square::F7.same_line_inclusive(a, b));
        assert!(Square::D5.same_line_inclusive(a, b));
        assert!(Square::C4.same_line_inclusive(a, b));
    }
}
