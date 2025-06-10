use crate::core::types::Piece;

use super::types::{Castling, Color, NB_PIECES};

pub struct FenState {
    pub bitboards: [u64; NB_PIECES],

    pub side_to_move: Color,
    pub castling: u8,
    // @TODO: en passant
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl FenState {
    #[rustfmt::skip]
    pub fn new() -> Self {
        let bitboards = [
            0x000000000000FF00,
            0x0000000000000042,
             0x0000000000000024,
               0x0000000000000081,
              0x0000000000000008,
               0x0000000000000010,
               0x00FF000000000000,
             0x4200000000000000,
             0x2400000000000000,
               0x8100000000000000,
              0x0800000000000000,
               0x1000000000000000,
        ];

        Self {
            bitboards,
            side_to_move: Color::White,
            castling: Castling::ALL.bits(),
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields".to_string());
        }

        let side_to_move = parse_side_to_move(parts[1])?;
        let castling = parse_castling(parts[2])?;
        let halfmove_clock = 0;
        let fullmove_number = 1;

        let mut state = Self { bitboards: [0; NB_PIECES], side_to_move, castling, halfmove_clock, fullmove_number };
        parse_board(parts[0], &mut state)?;
        Ok(state)
    }

    pub fn to_board_string(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mask = 1u64 << sq;
                let c = if self.bitboards[Piece::WBishop as usize] & mask != 0 {
                    'B'
                } else if self.bitboards[Piece::WKnight as usize] & mask != 0 {
                    'N'
                } else if self.bitboards[Piece::WPawn as usize] & mask != 0 {
                    'P'
                } else if self.bitboards[Piece::WRook as usize] & mask != 0 {
                    'R'
                } else if self.bitboards[Piece::WQueen as usize] & mask != 0 {
                    'Q'
                } else if self.bitboards[Piece::WKing as usize] & mask != 0 {
                    'K'
                } else if self.bitboards[Piece::BBishop as usize] & mask != 0 {
                    'b'
                } else if self.bitboards[Piece::BKnight as usize] & mask != 0 {
                    'n'
                } else if self.bitboards[Piece::BPawn as usize] & mask != 0 {
                    'p'
                } else if self.bitboards[Piece::BRook as usize] & mask != 0 {
                    'r'
                } else if self.bitboards[Piece::BQueen as usize] & mask != 0 {
                    'q'
                } else if self.bitboards[Piece::BKing as usize] & mask != 0 {
                    'k'
                } else {
                    '.'
                };
                s.push(c);
            }
        }
        s
    }

    pub fn to_string(&self, pad: bool) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            s.push((rank as u8 + b'1') as char);
            s.push(' ');
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mask = 1u64 << sq;
                let piece_char = if self.bitboards[Piece::WPawn as usize] & mask != 0 {
                    '♙'
                } else if self.bitboards[Piece::WKnight as usize] & mask != 0 {
                    '♘'
                } else if self.bitboards[Piece::WBishop as usize] & mask != 0 {
                    '♗'
                } else if self.bitboards[Piece::WRook as usize] & mask != 0 {
                    '♖'
                } else if self.bitboards[Piece::WQueen as usize] & mask != 0 {
                    '♕'
                } else if self.bitboards[Piece::WKing as usize] & mask != 0 {
                    '♔'
                } else if self.bitboards[Piece::BPawn as usize] & mask != 0 {
                    '♟'
                } else if self.bitboards[Piece::BKnight as usize] & mask != 0 {
                    '♞'
                } else if self.bitboards[Piece::BBishop as usize] & mask != 0 {
                    '♝'
                } else if self.bitboards[Piece::BRook as usize] & mask != 0 {
                    '♜'
                } else if self.bitboards[Piece::BQueen as usize] & mask != 0 {
                    '♛'
                } else if self.bitboards[Piece::BKing as usize] & mask != 0 {
                    '♚'
                } else {
                    '.'
                };

                if piece_char == '.' {
                    s.push('・');
                } else {
                    s.push(piece_char);
                    if pad {
                        s.push(' ');
                    }
                }
            }
            s.push('\n');
        }
        s.push_str("  ａｂｃｄｅｆｇｈ\n");
        s.push_str(format!("Side: {}\n", if self.side_to_move == Color::White { "White" } else { "Black" }).as_str());
        s.push_str(format!("Castling: {}\n", &castling_to_string(self.castling)).as_str());
        s.push_str(format!("Halfmove clock: {}\n", self.halfmove_clock).as_str());
        s.push_str(format!("Fullmove number: {}\n", self.fullmove_number).as_str());

        s
    }
}

fn castling_to_string(castling: u8) -> String {
    let mut result = String::new();
    for (i, c) in ['K', 'Q', 'k', 'q'].iter().enumerate() {
        if castling & (1 << i) != 0 {
            result.push(*c);
        }
    }
    if result.is_empty() { "-".to_string() } else { result }
}

fn parse_board(input: &str, state: &mut FenState) -> Result<(), String> {
    for (row, rank_str) in input.split('/').enumerate() {
        let mut file = 0;
        for c in rank_str.chars() {
            let rank = 7 - row as u8;
            let sq = (rank << 3) + file as u8;
            let mut inc = 1;
            match c {
                'p' => state.bitboards[Piece::BPawn as usize] |= 1 << sq,
                'r' => state.bitboards[Piece::BRook as usize] |= 1 << sq,
                'n' => state.bitboards[Piece::BKnight as usize] |= 1 << sq,
                'b' => state.bitboards[Piece::BBishop as usize] |= 1 << sq,
                'q' => state.bitboards[Piece::BQueen as usize] |= 1 << sq,
                'k' => state.bitboards[Piece::BKing as usize] |= 1 << sq,
                'P' => state.bitboards[Piece::WPawn as usize] |= 1 << sq,
                'R' => state.bitboards[Piece::WRook as usize] |= 1 << sq,
                'N' => state.bitboards[Piece::WKnight as usize] |= 1 << sq,
                'B' => state.bitboards[Piece::WBishop as usize] |= 1 << sq,
                'Q' => state.bitboards[Piece::WQueen as usize] |= 1 << sq,
                'K' => state.bitboards[Piece::WKing as usize] |= 1 << sq,
                '1'..='8' => inc = c.to_digit(10).unwrap(),
                _ => return Err(format!("Invalid character '{}' in board layout", c)),
            }

            file += inc as usize;
        }
        if file != 8 {
            return Err("Invalid board layout in FEN".to_string());
        }
    }

    Ok(())
}

fn parse_side_to_move(input: &str) -> Result<Color, String> {
    match input {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err("Invalid side to move in FEN".to_string()),
    }
}

fn parse_castling(input: &str) -> Result<u8, String> {
    if input == "-" {
        return Ok(0);
    }

    if input.len() > 4 {
        return Err("Invalid castling rights in FEN".to_string());
    }
    let mut castling = 0;
    for c in input.chars() {
        match c {
            'K' => castling |= Castling::WK.bits(),
            'Q' => castling |= Castling::WQ.bits(),
            'k' => castling |= Castling::BK.bits(),
            'q' => castling |= Castling::BQ.bits(),
            _ => return Err("Invalid castling rights in FEN".to_string()),
        }
    }

    Ok(castling)
}

pub fn occupancies(state: &FenState) -> [u64; 3] {
    let white_pieces = state.bitboards[Piece::WPawn as usize]
        | state.bitboards[Piece::WKnight as usize]
        | state.bitboards[Piece::WBishop as usize]
        | state.bitboards[Piece::WRook as usize]
        | state.bitboards[Piece::WQueen as usize]
        | state.bitboards[Piece::WKing as usize];
    let black_pieces = state.bitboards[Piece::BPawn as usize]
        | state.bitboards[Piece::BKnight as usize]
        | state.bitboards[Piece::BBishop as usize]
        | state.bitboards[Piece::BRook as usize]
        | state.bitboards[Piece::BQueen as usize]
        | state.bitboards[Piece::BKing as usize];
    [white_pieces, black_pieces, white_pieces | black_pieces]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = FenState::new();
        assert_eq!(state.bitboards[Piece::WPawn as usize], 0x000000000000FF00u64);
        assert_eq!(state.bitboards[Piece::BPawn as usize], 0x00FF000000000000u64);
        assert_eq!(state.bitboards[Piece::WRook as usize], 0x0000000000000081u64);
        assert_eq!(state.bitboards[Piece::BRook as usize], 0x8100000000000000u64);

        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.castling, Castling::ALL.bits());
        assert_eq!(state.halfmove_clock, 0);
        assert_eq!(state.fullmove_number, 1);
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let state = FenState::from_fen(fen).unwrap();
        assert_eq!(state.bitboards[Piece::WPawn as usize], 0x000000000000FF00u64);
        assert_eq!(state.bitboards[Piece::BPawn as usize], 0x00FF000000000000u64);
        assert_eq!(state.bitboards[Piece::WRook as usize], 0x0000000000000081u64);
        assert_eq!(state.bitboards[Piece::BRook as usize], 0x8100000000000000u64);

        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.castling, Castling::ALL.bits());
        assert_eq!(state.halfmove_clock, 0);
        assert_eq!(state.fullmove_number, 1);
    }

    #[test]
    fn test_parse_side_to_move() {
        assert_eq!(parse_side_to_move("w").unwrap(), Color::White);
        assert_eq!(parse_side_to_move("b").unwrap(), Color::Black);
        assert!(parse_side_to_move("-").is_err());
        assert!(parse_side_to_move("??").is_err());
    }

    #[test]
    fn test_parse_castling() {
        assert_eq!(
            parse_castling("KQkq").unwrap(),
            Castling::WK.bits() | Castling::WQ.bits() | Castling::BK.bits() | Castling::BQ.bits()
        );
        assert_eq!(parse_castling("KQ").unwrap(), Castling::WK.bits() | Castling::WQ.bits());
        assert_eq!(parse_castling("kq").unwrap(), Castling::BK.bits() | Castling::BQ.bits());
        assert_eq!(parse_castling("-").unwrap(), 0);
        assert!(parse_castling("X").is_err());
    }

    #[test]
    fn constructor_test() {
        let state = FenState::new();
        assert_eq!(state.to_board_string(), "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR");
    }

    #[test]
    fn parse_fen_test1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let state = FenState::from_fen(fen).unwrap();
        assert_eq!(state.to_board_string(), "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR");
    }

    #[test]
    fn parse_fen_test2() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        let state = FenState::from_fen(fen).unwrap();

        assert_eq!(state.to_board_string(), "r..q.rk.pp..bppp..n.pn....bp........P.....NP.N..PPQ..PPPR.B..RK.");
    }

    #[test]
    fn parse_fen_test3() {
        let fen = "r1bqk2r/pp1n1ppp/2pbpn2/8/3P4/2N1BN2/PPP2PPP/R2QKB1R w KQkq - 6 7";
        let state = FenState::from_fen(fen).unwrap();

        assert_eq!(state.to_board_string(), "r.bqk..rpp.n.ppp..pbpn.............P......N.BN..PPP..PPPR..QKB.R");
    }
}
