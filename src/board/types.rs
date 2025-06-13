use super::bitboard::BitBoard;
use std::fmt;

pub const FILE_A: u8 = 0;
pub const FILE_B: u8 = 1;
pub const FILE_C: u8 = 2;
pub const FILE_D: u8 = 3;
pub const FILE_E: u8 = 4;
pub const FILE_F: u8 = 5;
pub const FILE_G: u8 = 6;
pub const FILE_H: u8 = 7;

// Constants for ranks
pub const RANK_1: u8 = 0;
pub const RANK_2: u8 = 1;
pub const RANK_3: u8 = 2;
pub const RANK_4: u8 = 3;
pub const RANK_5: u8 = 4;
pub const RANK_6: u8 = 5;
pub const RANK_7: u8 = 6;
pub const RANK_8: u8 = 7;

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
// Constants for files

pub const BB_A1: BitBoard = Square::A1.to_bitboard();
pub const BB_A2: BitBoard = Square::A2.to_bitboard();
pub const BB_A3: BitBoard = Square::A3.to_bitboard();
pub const BB_A4: BitBoard = Square::A4.to_bitboard();
pub const BB_A5: BitBoard = Square::A5.to_bitboard();
pub const BB_A6: BitBoard = Square::A6.to_bitboard();
pub const BB_A7: BitBoard = Square::A7.to_bitboard();
pub const BB_A8: BitBoard = Square::A8.to_bitboard();

pub const BB_B1: BitBoard = Square::B1.to_bitboard();
pub const BB_B2: BitBoard = Square::B2.to_bitboard();
pub const BB_B3: BitBoard = Square::B3.to_bitboard();
pub const BB_B4: BitBoard = Square::B4.to_bitboard();
pub const BB_B5: BitBoard = Square::B5.to_bitboard();
pub const BB_B6: BitBoard = Square::B6.to_bitboard();
pub const BB_B7: BitBoard = Square::B7.to_bitboard();
pub const BB_B8: BitBoard = Square::B8.to_bitboard();

pub const BB_C1: BitBoard = Square::C1.to_bitboard();
pub const BB_C2: BitBoard = Square::C2.to_bitboard();
pub const BB_C3: BitBoard = Square::C3.to_bitboard();
pub const BB_C4: BitBoard = Square::C4.to_bitboard();
pub const BB_C5: BitBoard = Square::C5.to_bitboard();
pub const BB_C6: BitBoard = Square::C6.to_bitboard();
pub const BB_C7: BitBoard = Square::C7.to_bitboard();
pub const BB_C8: BitBoard = Square::C8.to_bitboard();

pub const BB_D1: BitBoard = Square::D1.to_bitboard();
pub const BB_D2: BitBoard = Square::D2.to_bitboard();
pub const BB_D3: BitBoard = Square::D3.to_bitboard();
pub const BB_D4: BitBoard = Square::D4.to_bitboard();
pub const BB_D5: BitBoard = Square::D5.to_bitboard();
pub const BB_D6: BitBoard = Square::D6.to_bitboard();
pub const BB_D7: BitBoard = Square::D7.to_bitboard();
pub const BB_D8: BitBoard = Square::D8.to_bitboard();

pub const BB_E1: BitBoard = Square::E1.to_bitboard();
pub const BB_E2: BitBoard = Square::E2.to_bitboard();
pub const BB_E3: BitBoard = Square::E3.to_bitboard();
pub const BB_E4: BitBoard = Square::E4.to_bitboard();
pub const BB_E5: BitBoard = Square::E5.to_bitboard();
pub const BB_E6: BitBoard = Square::E6.to_bitboard();
pub const BB_E7: BitBoard = Square::E7.to_bitboard();
pub const BB_E8: BitBoard = Square::E8.to_bitboard();

pub const BB_F1: BitBoard = Square::F1.to_bitboard();
pub const BB_F2: BitBoard = Square::F2.to_bitboard();
pub const BB_F3: BitBoard = Square::F3.to_bitboard();
pub const BB_F4: BitBoard = Square::F4.to_bitboard();
pub const BB_F5: BitBoard = Square::F5.to_bitboard();
pub const BB_F6: BitBoard = Square::F6.to_bitboard();
pub const BB_F7: BitBoard = Square::F7.to_bitboard();
pub const BB_F8: BitBoard = Square::F8.to_bitboard();

pub const BB_G1: BitBoard = Square::G1.to_bitboard();
pub const BB_G2: BitBoard = Square::G2.to_bitboard();
pub const BB_G3: BitBoard = Square::G3.to_bitboard();
pub const BB_G4: BitBoard = Square::G4.to_bitboard();
pub const BB_G5: BitBoard = Square::G5.to_bitboard();
pub const BB_G6: BitBoard = Square::G6.to_bitboard();
pub const BB_G7: BitBoard = Square::G7.to_bitboard();
pub const BB_G8: BitBoard = Square::G8.to_bitboard();

pub const BB_H1: BitBoard = Square::H1.to_bitboard();
pub const BB_H2: BitBoard = Square::H2.to_bitboard();
pub const BB_H3: BitBoard = Square::H3.to_bitboard();
pub const BB_H4: BitBoard = Square::H4.to_bitboard();
pub const BB_H5: BitBoard = Square::H5.to_bitboard();
pub const BB_H6: BitBoard = Square::H6.to_bitboard();
pub const BB_H7: BitBoard = Square::H7.to_bitboard();
pub const BB_H8: BitBoard = Square::H8.to_bitboard();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_square_test() {
        assert_eq!(Square::make(FILE_A, RANK_8), Square::A8);
        assert_eq!(Square::make(FILE_B, RANK_7), Square::B7);
        assert_eq!(Square::make(FILE_C, RANK_6), Square::C6);
        assert_eq!(Square::make(FILE_D, RANK_5), Square::D5);
        assert_eq!(Square::make(FILE_E, RANK_4), Square::E4);
        assert_eq!(Square::make(FILE_F, RANK_3), Square::F3);
        assert_eq!(Square::make(FILE_G, RANK_2), Square::G2);
        assert_eq!(Square::make(FILE_H, RANK_1), Square::H1);
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
}
