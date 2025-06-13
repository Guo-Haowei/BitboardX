pub mod fen {
    use crate::board::bitboard::BitBoard;
    use crate::board::moves::MoveFlags;
    use crate::board::piece::*;

    pub fn parse_board(input: &str) -> Result<[BitBoard; Piece::COUNT], &'static str> {
        let mut bitboards = [BitBoard::new(); Piece::COUNT];

        for (row, rank_str) in input.split('/').enumerate() {
            let mut file = 0;
            for c in rank_str.chars() {
                let rank = 7 - row as u8;
                let sq = (rank << 3) + file as u8;
                let mut inc = 1;
                match c {
                    'p' => bitboards[Piece::B_PAWN.as_usize()].set(sq),
                    'r' => bitboards[Piece::B_ROOK.as_usize()].set(sq),
                    'n' => bitboards[Piece::B_KNIGHT.as_usize()].set(sq),
                    'b' => bitboards[Piece::B_BISHOP.as_usize()].set(sq),
                    'q' => bitboards[Piece::B_QUEEN.as_usize()].set(sq),
                    'k' => bitboards[Piece::B_KING.as_usize()].set(sq),
                    'P' => bitboards[Piece::W_PAWN.as_usize()].set(sq),
                    'R' => bitboards[Piece::W_ROOK.as_usize()].set(sq),
                    'N' => bitboards[Piece::W_KNIGHT.as_usize()].set(sq),
                    'B' => bitboards[Piece::W_BISHOP.as_usize()].set(sq),
                    'Q' => bitboards[Piece::W_QUEEN.as_usize()].set(sq),
                    'K' => bitboards[Piece::W_KING.as_usize()].set(sq),
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

    pub fn calc_occupancies(bitboards: &[BitBoard; Piece::COUNT]) -> [BitBoard; 3] {
        let white_pieces = bitboards[Piece::W_PAWN.as_usize()]
            | bitboards[Piece::W_KNIGHT.as_usize()]
            | bitboards[Piece::W_BISHOP.as_usize()]
            | bitboards[Piece::W_ROOK.as_usize()]
            | bitboards[Piece::W_QUEEN.as_usize()]
            | bitboards[Piece::W_KING.as_usize()];
        let black_pieces = bitboards[Piece::B_PAWN.as_usize()]
            | bitboards[Piece::B_KNIGHT.as_usize()]
            | bitboards[Piece::B_BISHOP.as_usize()]
            | bitboards[Piece::B_ROOK.as_usize()]
            | bitboards[Piece::B_QUEEN.as_usize()]
            | bitboards[Piece::B_KING.as_usize()];
        [white_pieces, black_pieces, white_pieces | black_pieces]
    }

    #[cfg(test)]
    mod test {
        use super::*;

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
