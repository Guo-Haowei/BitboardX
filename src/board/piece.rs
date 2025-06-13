use crate::board::piece;

pub const COLOR_WHITE: u8 = 0;
pub const COLOR_BLACK: u8 = 1;
pub const COLOR_BOTH: u8 = 2;
pub const COLOR_NONE: u8 = COLOR_BOTH;

pub const fn get_opposite_color(color: u8) -> u8 {
    debug_assert!(color != COLOR_BOTH);
    return 1 - color;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

const NB_PIECE_TYPES: usize = 6;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Piece(u8);

impl Piece {
    // @TODO: rename
    pub const WPawn: Piece = Piece(0);
    pub const WKnight: Piece = Piece(1);
    pub const WBishop: Piece = Piece(2);
    pub const WRook: Piece = Piece(3);
    pub const WQueen: Piece = Piece(4);
    pub const WKing: Piece = Piece(5);
    pub const BPawn: Piece = Piece(6);
    pub const BKnight: Piece = Piece(7);
    pub const BBishop: Piece = Piece(8);
    pub const BRook: Piece = Piece(9);
    pub const BQueen: Piece = Piece(10);
    pub const BKing: Piece = Piece(11);
    pub const None: Piece = Piece(12);
    pub const COUNT: usize = Self::None.0 as usize;
    pub const W_START: u8 = Self::WPawn.0;
    pub const W_END: u8 = Self::WKing.0;
    pub const B_START: u8 = Self::BPawn.0;
    pub const B_END: u8 = Self::BKing.0;

    pub const fn color(&self) -> u8 {
        match self.0 {
            Self::W_START..=Self::W_END => COLOR_WHITE,
            Self::B_START..=Self::B_END => COLOR_BLACK,
            _ => 2, // None
        }
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn piece_type(&self) -> PieceType {
        debug_assert!(self.0 <= Self::COUNT as u8);
        if self.0 >= Self::COUNT as u8 {
            return PieceType::None;
        }

        let piece_type = self.0 % NB_PIECE_TYPES as u8;
        unsafe { std::mem::transmute(piece_type) }
    }

    pub fn get_piece(color: u8, piece_type: PieceType) -> Piece {
        debug_assert!(color < COLOR_BOTH);
        debug_assert!(piece_type != PieceType::None);
        unsafe { std::mem::transmute((color * NB_PIECE_TYPES as u8) + piece_type as u8) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_color() {
        assert_eq!(Piece::WPawn.color(), COLOR_WHITE);
        assert_eq!(Piece::WKnight.color(), COLOR_WHITE);
        assert_eq!(Piece::WBishop.color(), COLOR_WHITE);
        assert_eq!(Piece::WRook.color(), COLOR_WHITE);
        assert_eq!(Piece::WQueen.color(), COLOR_WHITE);
        assert_eq!(Piece::WKing.color(), COLOR_WHITE);
        assert_eq!(Piece::BPawn.color(), COLOR_BLACK);
        assert_eq!(Piece::BKnight.color(), COLOR_BLACK);
        assert_eq!(Piece::BBishop.color(), COLOR_BLACK);
        assert_eq!(Piece::BRook.color(), COLOR_BLACK);
        assert_eq!(Piece::BQueen.color(), COLOR_BLACK);
        assert_eq!(Piece::BKing.color(), COLOR_BLACK);
    }
}
