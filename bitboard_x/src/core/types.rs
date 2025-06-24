use std::fmt;

pub mod bitboard;
pub mod movement;
pub mod square;

pub use bitboard::BitBoard;
pub use movement::*;
pub use square::{File, Rank, Square};

const COLOR_WHITE: u8 = 0;
const COLOR_BLACK: u8 = 1;
const COLOR_BOTH: u8 = 2;
const COLOR_NONE: u8 = COLOR_BOTH;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color(u8);

impl Color {
    pub const WHITE: Color = Color(0);
    pub const BLACK: Color = Color(1);
    pub const BOTH: Color = Color(2);
    pub const NONE: Color = Color(COLOR_NONE);
    pub const COUNT: usize = 2;

    pub const fn new(color: u8) -> Color {
        debug_assert!(color < COLOR_BOTH);
        Color(color)
    }

    pub const fn is_white(&self) -> bool {
        self.0 == COLOR_WHITE
    }

    pub const fn is_black(&self) -> bool {
        self.0 == COLOR_BLACK
    }

    pub const fn flip(&self) -> Color {
        debug_assert!((self.is_white() ^ self.is_black()));
        Color(self.0 ^ 1)
    }

    pub fn parse(color: &str) -> Option<Color> {
        match color {
            "w" => Some(Self::WHITE),
            "b" => Some(Self::BLACK),
            _ => None,
        }
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

pub const fn get_opposite_color(color: u8) -> u8 {
    debug_assert!(color != COLOR_BOTH);
    return 1 - color;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PieceType(pub u8);

impl PieceType {
    pub const PAWN: PieceType = PieceType(0);
    pub const KNIGHT: PieceType = PieceType(1);
    pub const BISHOP: PieceType = PieceType(2);
    pub const ROOK: PieceType = PieceType(3);
    pub const QUEEN: PieceType = PieceType(4);
    pub const KING: PieceType = PieceType(5);
    pub const NONE: PieceType = PieceType(6);

    pub const COUNT: u8 = Self::NONE.0;

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece(u8);

impl Piece {
    pub const W_PAWN: Piece = Piece(0);
    pub const W_KNIGHT: Piece = Piece(1);
    pub const W_BISHOP: Piece = Piece(2);
    pub const W_ROOK: Piece = Piece(3);
    pub const W_QUEEN: Piece = Piece(4);
    pub const W_KING: Piece = Piece(5);
    pub const B_PAWN: Piece = Piece(6);
    pub const B_KNIGHT: Piece = Piece(7);
    pub const B_BISHOP: Piece = Piece(8);
    pub const B_ROOK: Piece = Piece(9);
    pub const B_QUEEN: Piece = Piece(10);
    pub const B_KING: Piece = Piece(11);
    pub const NONE: Piece = Piece(12);
    pub const COUNT: usize = Self::NONE.0 as usize;
    pub const W_START: u8 = Self::W_PAWN.0;
    pub const W_END: u8 = Self::W_KING.0;
    pub const B_START: u8 = Self::B_PAWN.0;
    pub const B_END: u8 = Self::B_KING.0;

    const NB_PIECE_TYPES: u8 = PieceType::COUNT as u8;

    pub const fn new(piece: u8) -> Piece {
        debug_assert!(piece <= Self::COUNT as u8);
        Piece(piece)
    }

    pub const fn color(&self) -> Color {
        debug_assert!(true);
        match self.0 {
            Self::W_START..=Self::W_END => Color::WHITE,
            Self::B_START..=Self::B_END => Color::BLACK,
            _ => Color::NONE, // None
        }
    }

    pub const fn get_type(&self) -> PieceType {
        debug_assert!(self.0 <= Self::COUNT as u8);
        if self.0 >= Self::COUNT as u8 {
            return PieceType::NONE;
        }

        let piece_type = self.0 % Self::NB_PIECE_TYPES as u8;
        unsafe { std::mem::transmute(piece_type) }
    }

    #[inline(always)]
    pub fn get_piece(color: Color, piece_type: PieceType) -> Piece {
        let color = color.as_u8();
        debug_assert!(color < COLOR_BOTH);
        debug_assert!(piece_type != PieceType::NONE);
        unsafe { std::mem::transmute((color * Self::NB_PIECE_TYPES as u8) + piece_type.0 as u8) }
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn parse(c: char) -> Option<Piece> {
        match c {
            'p' => Some(Piece::B_PAWN),
            'r' => Some(Piece::B_ROOK),
            'n' => Some(Piece::B_KNIGHT),
            'b' => Some(Piece::B_BISHOP),
            'q' => Some(Piece::B_QUEEN),
            'k' => Some(Piece::B_KING),
            'P' => Some(Piece::W_PAWN),
            'R' => Some(Piece::W_ROOK),
            'N' => Some(Piece::W_KNIGHT),
            'B' => Some(Piece::W_BISHOP),
            'Q' => Some(Piece::W_QUEEN),
            'K' => Some(Piece::W_KING),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match self.0 {
            0 => 'P',  // White Pawn
            1 => 'N',  // White Knight
            2 => 'B',  // White Bishop
            3 => 'R',  // White Rook
            4 => 'Q',  // White Queen
            5 => 'K',  // White King
            6 => 'p',  // Black Pawn
            7 => 'n',  // Black Knight
            8 => 'b',  // Black Bishop
            9 => 'r',  // Black Rook
            10 => 'q', // Black Queen
            11 => 'k', // Black King
            _ => '.',  // None
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            COLOR_WHITE => write!(f, "White"),
            COLOR_BLACK => write!(f, "Black"),
            _ => write!(f, "-"),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        assert!(Color::WHITE.is_white());
        assert!(!Color::WHITE.is_black());
        assert!(!Color::BLACK.is_white());
        assert!(Color::BLACK.is_black());
    }

    #[test]
    fn test_opponent_color() {
        assert_eq!(Color::WHITE.flip(), Color::BLACK);
        assert_eq!(Color::BLACK.flip(), Color::WHITE);
        // assert_eq!(Color::BOTH.opponent(), Color::NONE); // crash
        // assert_eq!(Color::NONE.opponent(), Color::NONE); // crash
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(Color::parse("w").unwrap(), Color::WHITE);
        assert_eq!(Color::parse("b").unwrap(), Color::BLACK);
        assert!(Color::parse("-").is_none());
        assert!(Color::parse("??").is_none());
    }

    #[test]
    fn test_piece_color() {
        assert_eq!(Piece::W_PAWN.color(), Color::WHITE);
        assert_eq!(Piece::W_KNIGHT.color(), Color::WHITE);
        assert_eq!(Piece::W_BISHOP.color(), Color::WHITE);
        assert_eq!(Piece::W_ROOK.color(), Color::WHITE);
        assert_eq!(Piece::W_QUEEN.color(), Color::WHITE);
        assert_eq!(Piece::W_KING.color(), Color::WHITE);
        assert_eq!(Piece::B_PAWN.color(), Color::BLACK);
        assert_eq!(Piece::B_KNIGHT.color(), Color::BLACK);
        assert_eq!(Piece::B_BISHOP.color(), Color::BLACK);
        assert_eq!(Piece::B_ROOK.color(), Color::BLACK);
        assert_eq!(Piece::B_QUEEN.color(), Color::BLACK);
        assert_eq!(Piece::B_KING.color(), Color::BLACK);
    }

    #[test]
    fn test_get_piece() {
        assert_eq!(Piece::get_piece(Color::WHITE, PieceType::PAWN), Piece::W_PAWN);
        assert_eq!(Piece::get_piece(Color::WHITE, PieceType::KNIGHT), Piece::W_KNIGHT);
        assert_eq!(Piece::get_piece(Color::WHITE, PieceType::BISHOP), Piece::W_BISHOP);
        assert_eq!(Piece::get_piece(Color::WHITE, PieceType::ROOK), Piece::W_ROOK);
        assert_eq!(Piece::get_piece(Color::WHITE, PieceType::QUEEN), Piece::W_QUEEN);
        assert_eq!(Piece::get_piece(Color::WHITE, PieceType::KING), Piece::W_KING);
        assert_eq!(Piece::get_piece(Color::BLACK, PieceType::PAWN), Piece::B_PAWN);
        assert_eq!(Piece::get_piece(Color::BLACK, PieceType::KNIGHT), Piece::B_KNIGHT);
        assert_eq!(Piece::get_piece(Color::BLACK, PieceType::BISHOP), Piece::B_BISHOP);
        assert_eq!(Piece::get_piece(Color::BLACK, PieceType::ROOK), Piece::B_ROOK);
        assert_eq!(Piece::get_piece(Color::BLACK, PieceType::QUEEN), Piece::B_QUEEN);
        assert_eq!(Piece::get_piece(Color::BLACK, PieceType::KING), Piece::B_KING);
    }
}
