pub mod fen {
    use crate::board::bitboard::BitBoard;
    use crate::board::moves::MoveFlags;
    use crate::board::types::*;

    pub fn parse_board(input: &str) -> Result<[BitBoard; NB_PIECES], &'static str> {
        let mut bitboards = [BitBoard::new(); NB_PIECES];

        for (row, rank_str) in input.split('/').enumerate() {
            let mut file = 0;
            for c in rank_str.chars() {
                let rank = 7 - row as u8;
                let sq = (rank << 3) + file as u8;
                let mut inc = 1;
                match c {
                    'p' => bitboards[Piece::BPawn as usize].set(sq),
                    'r' => bitboards[Piece::BRook as usize].set(sq),
                    'n' => bitboards[Piece::BKnight as usize].set(sq),
                    'b' => bitboards[Piece::BBishop as usize].set(sq),
                    'q' => bitboards[Piece::BQueen as usize].set(sq),
                    'k' => bitboards[Piece::BKing as usize].set(sq),
                    'P' => bitboards[Piece::WPawn as usize].set(sq),
                    'R' => bitboards[Piece::WRook as usize].set(sq),
                    'N' => bitboards[Piece::WKnight as usize].set(sq),
                    'B' => bitboards[Piece::WBishop as usize].set(sq),
                    'Q' => bitboards[Piece::WQueen as usize].set(sq),
                    'K' => bitboards[Piece::WKing as usize].set(sq),
                    '1'..='8' => inc = c.to_digit(10).unwrap(),
                    _ => return Err("Invalid character in board layout"),
                }

                file += inc as usize;
            }
            if file != 8 {
                return Err("Invalid board layout in FEN");
            }
        }

        Ok(bitboards)
    }

    pub fn parse_side_to_move(input: &str) -> Result<u8, &'static str> {
        match input {
            "w" => Ok(COLOR_WHITE),
            "b" => Ok(COLOR_BLACK),
            _ => Err("Invalid side to move in FEN"),
        }
    }

    pub fn parse_castling(input: &str) -> Result<u8, &'static str> {
        if input == "-" {
            return Ok(0);
        }

        if input.len() > 4 {
            return Err("Invalid castling rights");
        }
        let mut castling = 0;
        for c in input.chars() {
            match c {
                'K' => castling |= MoveFlags::K,
                'Q' => castling |= MoveFlags::Q,
                'k' => castling |= MoveFlags::k,
                'q' => castling |= MoveFlags::q,
                _ => return Err("Invalid castling rights"),
            }
        }

        Ok(castling)
    }

    pub fn parse_halfmove_clock(input: &str) -> Result<u32, &'static str> {
        input.parse().map_err(|_| "Invalid halfmove clock")
    }

    pub fn parse_fullmove_number(input: &str) -> Result<u32, &'static str> {
        input.parse().map_err(|_| "Invalid fullmove number")
    }

    pub fn calc_occupancies(bitboards: &[BitBoard; NB_PIECES]) -> [BitBoard; 3] {
        let white_pieces = bitboards[Piece::WPawn as usize]
            | bitboards[Piece::WKnight as usize]
            | bitboards[Piece::WBishop as usize]
            | bitboards[Piece::WRook as usize]
            | bitboards[Piece::WQueen as usize]
            | bitboards[Piece::WKing as usize];
        let black_pieces = bitboards[Piece::BPawn as usize]
            | bitboards[Piece::BKnight as usize]
            | bitboards[Piece::BBishop as usize]
            | bitboards[Piece::BRook as usize]
            | bitboards[Piece::BQueen as usize]
            | bitboards[Piece::BKing as usize];
        [white_pieces, black_pieces, white_pieces | black_pieces]
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_parse_side_to_move() {
            assert_eq!(parse_side_to_move("w").unwrap(), COLOR_WHITE);
            assert_eq!(parse_side_to_move("b").unwrap(), COLOR_BLACK);
            assert!(parse_side_to_move("-").is_err());
            assert!(parse_side_to_move("??").is_err());
        }

        #[test]
        fn test_parse_castling() {
            assert_eq!(parse_castling("KQkq").unwrap(), MoveFlags::KQkq);
            assert_eq!(parse_castling("KQ").unwrap(), MoveFlags::KQ);
            assert_eq!(parse_castling("kq").unwrap(), MoveFlags::kq);
            assert_eq!(parse_castling("-").unwrap(), 0);
            assert!(parse_castling("X").is_err());
        }
    }
}
