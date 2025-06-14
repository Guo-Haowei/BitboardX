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
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (file, rank) = self.file_rank();
        write!(f, "{}{}", (b'a' + file) as char, rank + 1)
    }
}
