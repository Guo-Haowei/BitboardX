use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FenState {
    pub white_pawn: u64,
    pub white_knight: u64,
    pub white_bishop: u64,
    pub white_rook: u64,
    pub white_queen: u64,
    pub white_king: u64,
    pub black_pawn: u64,
    pub black_knight: u64,
    pub black_bishop: u64,
    pub black_rook: u64,
    pub black_queen: u64,
    pub black_king: u64,

    pub side_to_move: Color,
    pub castling: u8,
    // @TODO: en passant
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

#[wasm_bindgen]
impl FenState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            white_pawn: 0x000000000000FF00,
            white_knight: 0x0000000000000042,
            white_bishop: 0x0000000000000024,
            white_rook: 0x0000000000000081,
            white_queen: 0x0000000000000008,
            white_king: 0x0000000000000010,
            black_pawn: 0x00FF000000000000,
            black_knight: 0x4200000000000000,
            black_bishop: 0x2400000000000000,
            black_rook: 0x8100000000000000,
            black_queen: 0x0800000000000000,
            black_king: 0x10000000000000,
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

        let mut state = Self {
            white_pawn: 0,
            white_knight: 0,
            white_bishop: 0,
            white_rook: 0,
            white_queen: 0,
            white_king: 0,
            black_pawn: 0,
            black_knight: 0,
            black_bishop: 0,
            black_rook: 0,
            black_queen: 0,
            black_king: 0,
            side_to_move,
            castling,
            halfmove_clock,
            fullmove_number,
        };
        parse_board(parts[0], &mut state)?;
        Ok(state)
    }

    pub fn to_board_string(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mask = 1u64 << sq;
                let c = if self.white_bishop & mask != 0 {
                    'B'
                } else if self.white_knight & mask != 0 {
                    'N'
                } else if self.white_pawn & mask != 0 {
                    'P'
                } else if self.white_rook & mask != 0 {
                    'R'
                } else if self.white_queen & mask != 0 {
                    'Q'
                } else if self.white_king & mask != 0 {
                    'K'
                } else if self.black_bishop & mask != 0 {
                    'b'
                } else if self.black_knight & mask != 0 {
                    'n'
                } else if self.black_pawn & mask != 0 {
                    'p'
                } else if self.black_rook & mask != 0 {
                    'r'
                } else if self.black_queen & mask != 0 {
                    'q'
                } else if self.black_king & mask != 0 {
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
                let piece_char = if self.white_pawn & mask != 0 {
                    '♙'
                } else if self.white_knight & mask != 0 {
                    '♘'
                } else if self.white_bishop & mask != 0 {
                    '♗'
                } else if self.white_rook & mask != 0 {
                    '♖'
                } else if self.white_queen & mask != 0 {
                    '♕'
                } else if self.white_king & mask != 0 {
                    '♔'
                } else if self.black_pawn & mask != 0 {
                    '♟'
                } else if self.black_knight & mask != 0 {
                    '♞'
                } else if self.black_bishop & mask != 0 {
                    '♝'
                } else if self.black_rook & mask != 0 {
                    '♜'
                } else if self.black_queen & mask != 0 {
                    '♛'
                } else if self.black_king & mask != 0 {
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
                'p' => state.black_pawn |= 1 << sq,
                'r' => state.black_rook |= 1 << sq,
                'n' => state.black_knight |= 1 << sq,
                'b' => state.black_bishop |= 1 << sq,
                'q' => state.black_queen |= 1 << sq,
                'k' => state.black_king |= 1 << sq,
                'P' => state.white_pawn |= 1 << sq,
                'R' => state.white_rook |= 1 << sq,
                'N' => state.white_knight |= 1 << sq,
                'B' => state.white_bishop |= 1 << sq,
                'Q' => state.white_queen |= 1 << sq,
                'K' => state.white_king |= 1 << sq,
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
    let white_pieces = state.white_pawn
        | state.white_knight
        | state.white_bishop
        | state.white_rook
        | state.white_queen
        | state.white_king;
    let black_pieces = state.black_pawn
        | state.black_knight
        | state.black_bishop
        | state.black_rook
        | state.black_queen
        | state.black_king;
    [white_pieces, black_pieces, white_pieces | black_pieces]
}

// @TODO: use array instead of Vec
pub fn to_vec(state: &FenState) -> Vec<u64> {
    vec![
        state.white_pawn,
        state.white_knight,
        state.white_bishop,
        state.white_rook,
        state.white_queen,
        state.white_king,
        state.black_pawn,
        state.black_knight,
        state.black_bishop,
        state.black_rook,
        state.black_queen,
        state.black_king,
    ]
}

// @TODO: use array instead of Vec
pub fn to_mut_vec(state: &mut FenState) -> Vec<&mut u64> {
    vec![
        &mut state.white_pawn,
        &mut state.white_knight,
        &mut state.white_bishop,
        &mut state.white_rook,
        &mut state.white_queen,
        &mut state.white_king,
        &mut state.black_pawn,
        &mut state.black_knight,
        &mut state.black_bishop,
        &mut state.black_rook,
        &mut state.black_queen,
        &mut state.black_king,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = FenState::new();
        assert_eq!(state.white_pawn, 0x000000000000FF00u64);
        assert_eq!(state.black_pawn, 0x00FF000000000000u64);
        assert_eq!(state.white_rook, 0x0000000000000081u64);
        assert_eq!(state.black_rook, 0x8100000000000000u64);

        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.castling, Castling::ALL.bits());
        assert_eq!(state.halfmove_clock, 0);
        assert_eq!(state.fullmove_number, 1);
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let state = FenState::from_fen(fen).unwrap();
        assert_eq!(state.white_pawn, 0x000000000000FF00u64);
        assert_eq!(state.black_pawn, 0x00FF000000000000u64);
        assert_eq!(state.white_rook, 0x0000000000000081u64);
        assert_eq!(state.black_rook, 0x8100000000000000u64);

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
