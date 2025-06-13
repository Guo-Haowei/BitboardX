use super::bitboard::BitBoard;

pub const fn make_square(file: u8, rank: u8) -> u8 {
    (rank << 3) + file
}

pub fn get_file_rank(square: u8) -> (u8, u8) {
    assert!(square < 64);
    let file = square & 0b111;
    let rank = square >> 3;
    (file, rank)
}

// Constants for files
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

// Constants for squares
pub const SQ_A1: u8 = 0u8;
pub const SQ_B1: u8 = 1u8;
pub const SQ_C1: u8 = 2u8;
pub const SQ_D1: u8 = 3u8;
pub const SQ_E1: u8 = 4u8;
pub const SQ_F1: u8 = 5u8;
pub const SQ_G1: u8 = 6u8;
pub const SQ_H1: u8 = 7u8;

pub const SQ_A2: u8 = 8u8;
pub const SQ_B2: u8 = 9u8;
pub const SQ_C2: u8 = 10u8;
pub const SQ_D2: u8 = 11u8;
pub const SQ_E2: u8 = 12u8;
pub const SQ_F2: u8 = 13u8;
pub const SQ_G2: u8 = 14u8;
pub const SQ_H2: u8 = 15u8;

pub const SQ_A3: u8 = 16u8;
pub const SQ_B3: u8 = 17u8;
pub const SQ_C3: u8 = 18u8;
pub const SQ_D3: u8 = 19u8;
pub const SQ_E3: u8 = 20u8;
pub const SQ_F3: u8 = 21u8;
pub const SQ_G3: u8 = 22u8;
pub const SQ_H3: u8 = 23u8;

pub const SQ_A4: u8 = 24u8;
pub const SQ_B4: u8 = 25u8;
pub const SQ_C4: u8 = 26u8;
pub const SQ_D4: u8 = 27u8;
pub const SQ_E4: u8 = 28u8;
pub const SQ_F4: u8 = 29u8;
pub const SQ_G4: u8 = 30u8;
pub const SQ_H4: u8 = 31u8;

pub const SQ_A5: u8 = 32u8;
pub const SQ_B5: u8 = 33u8;
pub const SQ_C5: u8 = 34u8;
pub const SQ_D5: u8 = 35u8;
pub const SQ_E5: u8 = 36u8;
pub const SQ_F5: u8 = 37u8;
pub const SQ_G5: u8 = 38u8;
pub const SQ_H5: u8 = 39u8;

pub const SQ_A6: u8 = 40u8;
pub const SQ_B6: u8 = 41u8;
pub const SQ_C6: u8 = 42u8;
pub const SQ_D6: u8 = 43u8;
pub const SQ_E6: u8 = 44u8;
pub const SQ_F6: u8 = 45u8;
pub const SQ_G6: u8 = 46u8;
pub const SQ_H6: u8 = 47u8;

pub const SQ_A7: u8 = 48u8;
pub const SQ_B7: u8 = 49u8;
pub const SQ_C7: u8 = 50u8;
pub const SQ_D7: u8 = 51u8;
pub const SQ_E7: u8 = 52u8;
pub const SQ_F7: u8 = 53u8;
pub const SQ_G7: u8 = 54u8;
pub const SQ_H7: u8 = 55u8;

pub const SQ_A8: u8 = 56u8;
pub const SQ_B8: u8 = 57u8;
pub const SQ_C8: u8 = 58u8;
pub const SQ_D8: u8 = 59u8;
pub const SQ_E8: u8 = 60u8;
pub const SQ_F8: u8 = 61u8;
pub const SQ_G8: u8 = 62u8;
pub const SQ_H8: u8 = 63u8;

pub const BB_A1: BitBoard = BitBoard::from_bit(SQ_A1);
pub const BB_A2: BitBoard = BitBoard::from_bit(SQ_A2);
pub const BB_A3: BitBoard = BitBoard::from_bit(SQ_A3);
pub const BB_A4: BitBoard = BitBoard::from_bit(SQ_A4);
pub const BB_A5: BitBoard = BitBoard::from_bit(SQ_A5);
pub const BB_A6: BitBoard = BitBoard::from_bit(SQ_A6);
pub const BB_A7: BitBoard = BitBoard::from_bit(SQ_A7);
pub const BB_A8: BitBoard = BitBoard::from_bit(SQ_A8);

pub const BB_B1: BitBoard = BitBoard::from_bit(SQ_B1);
pub const BB_B2: BitBoard = BitBoard::from_bit(SQ_B2);
pub const BB_B3: BitBoard = BitBoard::from_bit(SQ_B3);
pub const BB_B4: BitBoard = BitBoard::from_bit(SQ_B4);
pub const BB_B5: BitBoard = BitBoard::from_bit(SQ_B5);
pub const BB_B6: BitBoard = BitBoard::from_bit(SQ_B6);
pub const BB_B7: BitBoard = BitBoard::from_bit(SQ_B7);
pub const BB_B8: BitBoard = BitBoard::from_bit(SQ_B8);

pub const BB_C1: BitBoard = BitBoard::from_bit(SQ_C1);
pub const BB_C2: BitBoard = BitBoard::from_bit(SQ_C2);
pub const BB_C3: BitBoard = BitBoard::from_bit(SQ_C3);
pub const BB_C4: BitBoard = BitBoard::from_bit(SQ_C4);
pub const BB_C5: BitBoard = BitBoard::from_bit(SQ_C5);
pub const BB_C6: BitBoard = BitBoard::from_bit(SQ_C6);
pub const BB_C7: BitBoard = BitBoard::from_bit(SQ_C7);
pub const BB_C8: BitBoard = BitBoard::from_bit(SQ_C8);

pub const BB_D1: BitBoard = BitBoard::from_bit(SQ_D1);
pub const BB_D2: BitBoard = BitBoard::from_bit(SQ_D2);
pub const BB_D3: BitBoard = BitBoard::from_bit(SQ_D3);
pub const BB_D4: BitBoard = BitBoard::from_bit(SQ_D4);
pub const BB_D5: BitBoard = BitBoard::from_bit(SQ_D5);
pub const BB_D6: BitBoard = BitBoard::from_bit(SQ_D6);
pub const BB_D7: BitBoard = BitBoard::from_bit(SQ_D7);
pub const BB_D8: BitBoard = BitBoard::from_bit(SQ_D8);

pub const BB_E1: BitBoard = BitBoard::from_bit(SQ_E1);
pub const BB_E2: BitBoard = BitBoard::from_bit(SQ_E2);
pub const BB_E3: BitBoard = BitBoard::from_bit(SQ_E3);
pub const BB_E4: BitBoard = BitBoard::from_bit(SQ_E4);
pub const BB_E5: BitBoard = BitBoard::from_bit(SQ_E5);
pub const BB_E6: BitBoard = BitBoard::from_bit(SQ_E6);
pub const BB_E7: BitBoard = BitBoard::from_bit(SQ_E7);
pub const BB_E8: BitBoard = BitBoard::from_bit(SQ_E8);

pub const BB_F1: BitBoard = BitBoard::from_bit(SQ_F1);
pub const BB_F2: BitBoard = BitBoard::from_bit(SQ_F2);
pub const BB_F3: BitBoard = BitBoard::from_bit(SQ_F3);
pub const BB_F4: BitBoard = BitBoard::from_bit(SQ_F4);
pub const BB_F5: BitBoard = BitBoard::from_bit(SQ_F5);
pub const BB_F6: BitBoard = BitBoard::from_bit(SQ_F6);
pub const BB_F7: BitBoard = BitBoard::from_bit(SQ_F7);
pub const BB_F8: BitBoard = BitBoard::from_bit(SQ_F8);

pub const BB_G1: BitBoard = BitBoard::from_bit(SQ_G1);
pub const BB_G2: BitBoard = BitBoard::from_bit(SQ_G2);
pub const BB_G3: BitBoard = BitBoard::from_bit(SQ_G3);
pub const BB_G4: BitBoard = BitBoard::from_bit(SQ_G4);
pub const BB_G5: BitBoard = BitBoard::from_bit(SQ_G5);
pub const BB_G6: BitBoard = BitBoard::from_bit(SQ_G6);
pub const BB_G7: BitBoard = BitBoard::from_bit(SQ_G7);
pub const BB_G8: BitBoard = BitBoard::from_bit(SQ_G8);

pub const BB_H1: BitBoard = BitBoard::from_bit(SQ_H1);
pub const BB_H2: BitBoard = BitBoard::from_bit(SQ_H2);
pub const BB_H3: BitBoard = BitBoard::from_bit(SQ_H3);
pub const BB_H4: BitBoard = BitBoard::from_bit(SQ_H4);
pub const BB_H5: BitBoard = BitBoard::from_bit(SQ_H5);
pub const BB_H6: BitBoard = BitBoard::from_bit(SQ_H6);
pub const BB_H7: BitBoard = BitBoard::from_bit(SQ_H7);
pub const BB_H8: BitBoard = BitBoard::from_bit(SQ_H8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_square_test() {
        assert_eq!(make_square(FILE_A, RANK_8), SQ_A8);
        assert_eq!(make_square(FILE_B, RANK_7), SQ_B7);
        assert_eq!(make_square(FILE_C, RANK_6), SQ_C6);
        assert_eq!(make_square(FILE_D, RANK_5), SQ_D5);
        assert_eq!(make_square(FILE_E, RANK_4), SQ_E4);
        assert_eq!(make_square(FILE_F, RANK_3), SQ_F3);
        assert_eq!(make_square(FILE_G, RANK_2), SQ_G2);
        assert_eq!(make_square(FILE_H, RANK_1), SQ_H1);
    }

    #[test]
    fn get_file_rank_test() {
        assert_eq!(get_file_rank(SQ_A8), (FILE_A, RANK_8));
        assert_eq!(get_file_rank(SQ_B7), (FILE_B, RANK_7));
        assert_eq!(get_file_rank(SQ_C6), (FILE_C, RANK_6));
        assert_eq!(get_file_rank(SQ_D5), (FILE_D, RANK_5));
        assert_eq!(get_file_rank(SQ_E4), (FILE_E, RANK_4));
        assert_eq!(get_file_rank(SQ_F3), (FILE_F, RANK_3));
        assert_eq!(get_file_rank(SQ_G2), (FILE_G, RANK_2));
        assert_eq!(get_file_rank(SQ_H1), (FILE_H, RANK_1));
    }
}
