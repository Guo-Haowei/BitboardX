use super::super::utils;
use super::bitboard::BitBoard;
use paste::paste;
use std::fmt;

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
pub struct Square(pub u8);

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

    // only returns true if square is between A and B
    pub fn same_line(&self, a: Square, b: Square) -> bool {
        let (ax, ay) = a.file_rank();
        let (bx, by) = b.file_rank();
        let (cx, cy) = self.file_rank();

        // Shoelace Formula (also called the Surveyor's Formula) for the area of a triangle in 2D space.
        // area = [ Ax * (By - Cy) + Bx * (Cy - Ay) + Cx * (Ay - By) ] / 2
        // but we only cares about the sign of the area, so we can skip the division by 2.
        let two_signed_area = ax as i32 * (by as i32 - cy as i32)
            + bx as i32 * (cy as i32 - ay as i32)
            + cx as i32 * (ay as i32 - by as i32);

        if two_signed_area != 0 {
            return false;
        }

        let (x_min, x_max) = utils::min_max(ax, bx);
        let (y_min, y_max) = utils::min_max(ay, by);
        let between_x = cx >= x_min && cx <= x_max;
        let between_y = cy >= y_min && cy <= y_max;

        between_x && between_y
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (file, rank) = self.file_rank();
        write!(f, "{}{}", (b'a' + file) as char, rank + 1)
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
        assert!(!a.same_line(b, c));
        assert!(b.same_line(a, c));
        assert!(!c.same_line(a, b));

        let a = Square::A1;
        let b = Square::B1;
        let c = Square::C3;
        assert!(!a.same_line(b, c));
        assert!(!b.same_line(a, c));
        assert!(!c.same_line(a, b));
    }

    #[test]
    fn same_line_overlapping() {
        let a = Square::B2;
        let b = Square::B2;
        let c = Square::D8;
        assert!(!c.same_line(a, b));
        assert!(a.same_line(b, c));
        assert!(b.same_line(a, c));
    }

    #[test]
    fn same_line_horizontal() {
        let a = Square::C1;
        let b = Square::C2;
        let c = Square::C5;
        assert!(!a.same_line(b, c));
        assert!(b.same_line(a, c));
        assert!(!c.same_line(a, b));
    }

    #[test]
    fn same_line_vertical() {
        let a = Square::A1;
        let b = Square::A8;
        let c = Square::A3;
        assert!(!a.same_line(b, c));
        assert!(!b.same_line(a, c));
        assert!(c.same_line(a, b));
    }

    #[test]
    fn more_same_line_test() {
        let a = Square::G8;
        let b = Square::B3;

        assert!(Square::F7.same_line(a, b));
        assert!(Square::D5.same_line(a, b));
        assert!(Square::C4.same_line(a, b));
    }
}
