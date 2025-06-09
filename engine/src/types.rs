use bitflags::bitflags;

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum Color {
    White,
    Black,
    Both,
}

#[rustfmt::skip]
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    Count,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    Count,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
    Count,
}

pub fn make_square(f: File, r: Rank) -> Square {
    let raw = ((r as u8) << 3) + f as u8;
    return unsafe { std::mem::transmute(raw) };
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Direction: u8 {
        const N = 1 << 0;
        const S = 1 << 1;
        const W = 1 << 2;
        const E = 1 << 3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_square_test() {
        assert_eq!(make_square(File::A, Rank::R8), Square::A8);
        assert_eq!(make_square(File::B, Rank::R7), Square::B7);
        assert_eq!(make_square(File::C, Rank::R6), Square::C6);
        assert_eq!(make_square(File::D, Rank::R5), Square::D5);
        assert_eq!(make_square(File::E, Rank::R4), Square::E4);
        assert_eq!(make_square(File::F, Rank::R3), Square::F3);
        assert_eq!(make_square(File::G, Rank::R2), Square::G2);
        assert_eq!(make_square(File::H, Rank::R1), Square::H1);
    }
}
