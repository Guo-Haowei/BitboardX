use crate::core::types::*;

pub fn parse_board(input: &str) -> Result<[BitBoard; Piece::COUNT], &'static str> {
    let mut bitboards = [BitBoard::new(); Piece::COUNT];

    for (row, rank_str) in input.split('/').enumerate() {
        let mut file = 0;
        for c in rank_str.chars() {
            let rank = 7 - row as u8;
            let sq = (rank << 3) + file as u8;
            let mut inc = 1;
            if let Some(piece) = Piece::parse(c) {
                bitboards[piece.as_usize()].set(sq);
            } else {
                match c {
                    '1'..='8' => inc = c.to_digit(10).unwrap(),
                    _ => return Err("Invalid character in board layout"),
                }
            }

            file += inc as usize;
        }
        if file != 8 {
            return Err("Invalid board layout in FEN");
        }
    }

    Ok(bitboards)
}

pub fn dump_board(bitboards: &[BitBoard; Piece::COUNT]) -> String {
    let mut s = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0;

        for file in 0..8 {
            let sq = rank * 8 + file;

            let piece_char = (Piece::W_PAWN.as_usize()..Piece::COUNT)
                .find(|&p| bitboards[p].test(sq))
                .map(|p| Piece::new(p as u8).to_char());

            match piece_char {
                Some(c) => {
                    if empty > 0 {
                        s.push((b'0' + empty) as char);
                        empty = 0;
                    }
                    s.push(c);
                }
                None => {
                    empty += 1;
                }
            }
        }
        if empty > 0 {
            s.push((b'0' + empty) as char);
        }
        s.push('/');
    }

    s.pop(); // Remove the last '/'
    s
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
            'K' => castling |= CastlingRight::K,
            'Q' => castling |= CastlingRight::Q,
            'k' => castling |= CastlingRight::k,
            'q' => castling |= CastlingRight::q,
            _ => return Err("Invalid castling rights"),
        }
    }

    Ok(castling)
}

pub fn dump_castling(castling: u8) -> String {
    let mut result = String::new();
    for (i, c) in ['K', 'Q', 'k', 'q'].iter().enumerate() {
        if castling & (1 << i) != 0 {
            result.push(*c);
        }
    }
    if result.is_empty() { "-".to_string() } else { result }
}

pub fn parse_en_passant(input: &str) -> Option<Option<Square>> {
    if input == "-" {
        return Some(None);
    }

    if input.len() == 2 {
        let file = File(input.chars().nth(0)? as u8 - b'a');
        let rank = Rank(input.chars().nth(1)? as u8 - b'1');
        match rank {
            Rank::_3 | Rank::_6 if file <= File::H => {
                return Some(Some(Square::make(file, rank)));
            }
            _ => {}
        }
    }

    None
}

pub fn parse_halfmove_clock(input: &str) -> Result<u32, &'static str> {
    input.parse().map_err(|_| "Invalid halfmove clock")
}

pub fn parse_fullmove_number(input: &str) -> Result<u32, &'static str> {
    input.parse().map_err(|_| "Invalid fullmove number")
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_castling() {
        assert_eq!(parse_castling("KQkq").unwrap(), CastlingRight::KQkq);
        assert_eq!(parse_castling("KQ").unwrap(), CastlingRight::KQ);
        assert_eq!(parse_castling("kq").unwrap(), CastlingRight::kq);
        assert_eq!(parse_castling("-").unwrap(), 0);
        assert!(parse_castling("X").is_err());
    }
}
