use super::board::*;
use super::types::*;

pub fn parse_square(input: &str) -> Option<Square> {
    if input.len() != 2 {
        return None;
    }

    let file = input.chars().nth(0)? as u8 - b'a';
    let rank = input.chars().nth(1)? as u8 - b'1';

    if file > 7 || rank > 7 {
        return None;
    }

    Some(Square::make(file, rank))
}

pub fn parse_move(input: &str) -> Option<(Square, Square)> {
    if input.len() != 4 {
        return None;
    }

    let from = parse_square(&input[0..2])?;
    let to = parse_square(&input[2..4])?;

    Some((from, to))
}

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

pub fn parse_en_passant(input: &str) -> Result<Option<Square>, &'static str> {
    if input == "-" {
        return Ok(None);
    }

    match parse_square(input) {
        Some(square) => {
            let (_file, rank) = square.file_rank();
            match rank {
                RANK_3 | RANK_6 => {
                    return Ok(Some(square));
                }
                _ => {
                    return Err("Invalid en passant square: must be on rank 3 or 6");
                }
            }
        }
        None => Err("Invalid en passant square"),
    }
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

    #[test]
    fn test_parse_move() {
        assert_eq!(parse_move("e2e4"), Some((Square::E2, Square::E4)));
        assert_eq!(parse_move("a7a8"), Some((Square::A7, Square::A8)));
        assert_eq!(parse_move("h1h2"), Some((Square::H1, Square::H2)));
        assert_eq!(parse_move("d4d5"), Some((Square::D4, Square::D5)));
        assert_eq!(parse_move("z1z2"), None);
        assert_eq!(parse_move("e9e4"), None);
        assert_eq!(parse_move("e2e"), None);
    }
}
