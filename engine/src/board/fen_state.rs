use crate::board::types::get_opposite_color;

use super::bitboard::BitBoard;
use super::types::{Castling, Color, NB_PIECES, Piece};

pub struct FenState {
    pub bitboards: [BitBoard; NB_PIECES],

    pub side_to_move: Color,
    pub castling: u8,
    // @TODO: en passant
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl FenState {
    pub fn new() -> Self {
        let bitboards = [
            BitBoard::from(0x000000000000FF00), // White Pawns
            BitBoard::from(0x0000000000000042), // White Knights
            BitBoard::from(0x0000000000000024), // White Bishops
            BitBoard::from(0x0000000000000081), // White Rooks
            BitBoard::from(0x0000000000000008), // White Queens
            BitBoard::from(0x0000000000000010), // White King
            BitBoard::from(0x00FF000000000000), // Black Pawns
            BitBoard::from(0x4200000000000000), // Black Knights
            BitBoard::from(0x2400000000000000), // Black Bishops
            BitBoard::from(0x8100000000000000), // Black Rooks
            BitBoard::from(0x0800000000000000), // Black Queens
            BitBoard::from(0x1000000000000000), // Black King
        ];

        Self {
            bitboards,
            side_to_move: Color::White,
            castling: Castling::ALL.bits(),
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn from(
        board: &str,
        side_to_move: &str,
        castling: &str,
        _en_passant: &str,
        _half: &str,
        _full: &str,
    ) -> Result<Self, String> {
        let side_to_move = parse_side_to_move(side_to_move)?;
        let castling = parse_castling(castling)?;
        let halfmove_clock = 0;
        let fullmove_number = 1;

        let mut state =
            Self { bitboards: [BitBoard::new(); NB_PIECES], side_to_move, castling, halfmove_clock, fullmove_number };
        parse_board(board, &mut state)?;
        Ok(state)
    }

    pub fn change_side(&mut self) {
        self.side_to_move = get_opposite_color(self.side_to_move);
    }

    pub fn to_board_string(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let c = if self.bitboards[Piece::WBishop as usize].has_bit(sq) {
                    'B'
                } else if self.bitboards[Piece::WKnight as usize].has_bit(sq) {
                    'N'
                } else if self.bitboards[Piece::WPawn as usize].has_bit(sq) {
                    'P'
                } else if self.bitboards[Piece::WRook as usize].has_bit(sq) {
                    'R'
                } else if self.bitboards[Piece::WQueen as usize].has_bit(sq) {
                    'Q'
                } else if self.bitboards[Piece::WKing as usize].has_bit(sq) {
                    'K'
                } else if self.bitboards[Piece::BBishop as usize].has_bit(sq) {
                    'b'
                } else if self.bitboards[Piece::BKnight as usize].has_bit(sq) {
                    'n'
                } else if self.bitboards[Piece::BPawn as usize].has_bit(sq) {
                    'p'
                } else if self.bitboards[Piece::BRook as usize].has_bit(sq) {
                    'r'
                } else if self.bitboards[Piece::BQueen as usize].has_bit(sq) {
                    'q'
                } else if self.bitboards[Piece::BKing as usize].has_bit(sq) {
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
                let piece_char = if self.bitboards[Piece::WPawn as usize].has_bit(sq) {
                    '♙'
                } else if self.bitboards[Piece::WKnight as usize].has_bit(sq) {
                    '♘'
                } else if self.bitboards[Piece::WBishop as usize].has_bit(sq) {
                    '♗'
                } else if self.bitboards[Piece::WRook as usize].has_bit(sq) {
                    '♖'
                } else if self.bitboards[Piece::WQueen as usize].has_bit(sq) {
                    '♕'
                } else if self.bitboards[Piece::WKing as usize].has_bit(sq) {
                    '♔'
                } else if self.bitboards[Piece::BPawn as usize].has_bit(sq) {
                    '♟'
                } else if self.bitboards[Piece::BKnight as usize].has_bit(sq) {
                    '♞'
                } else if self.bitboards[Piece::BBishop as usize].has_bit(sq) {
                    '♝'
                } else if self.bitboards[Piece::BRook as usize].has_bit(sq) {
                    '♜'
                } else if self.bitboards[Piece::BQueen as usize].has_bit(sq) {
                    '♛'
                } else if self.bitboards[Piece::BKing as usize].has_bit(sq) {
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
                'p' => state.bitboards[Piece::BPawn as usize].set_bit(sq),
                'r' => state.bitboards[Piece::BRook as usize].set_bit(sq),
                'n' => state.bitboards[Piece::BKnight as usize].set_bit(sq),
                'b' => state.bitboards[Piece::BBishop as usize].set_bit(sq),
                'q' => state.bitboards[Piece::BQueen as usize].set_bit(sq),
                'k' => state.bitboards[Piece::BKing as usize].set_bit(sq),
                'P' => state.bitboards[Piece::WPawn as usize].set_bit(sq),
                'R' => state.bitboards[Piece::WRook as usize].set_bit(sq),
                'N' => state.bitboards[Piece::WKnight as usize].set_bit(sq),
                'B' => state.bitboards[Piece::WBishop as usize].set_bit(sq),
                'Q' => state.bitboards[Piece::WQueen as usize].set_bit(sq),
                'K' => state.bitboards[Piece::WKing as usize].set_bit(sq),
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

pub fn occupancies(state: &FenState) -> [BitBoard; 3] {
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
        assert!(state.bitboards[Piece::WPawn as usize].equal(0x000000000000FF00u64));
        assert!(state.bitboards[Piece::BPawn as usize].equal(0x00FF000000000000u64));
        assert!(state.bitboards[Piece::WRook as usize].equal(0x0000000000000081u64));
        assert!(state.bitboards[Piece::BRook as usize].equal(0x8100000000000000u64));

        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.castling, Castling::ALL.bits());
        assert_eq!(state.halfmove_clock, 0);
        assert_eq!(state.fullmove_number, 1);
    }

    #[test]
    fn test_from_fen() {
        let state = FenState::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", "w", "KQkq", "-", "0", "1").unwrap();
        assert!(state.bitboards[Piece::WPawn as usize].equal(0x000000000000FF00u64));
        assert!(state.bitboards[Piece::BPawn as usize].equal(0x00FF000000000000u64));
        assert!(state.bitboards[Piece::WRook as usize].equal(0x0000000000000081u64));
        assert!(state.bitboards[Piece::BRook as usize].equal(0x8100000000000000u64));

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
        let state = FenState::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", "w", "KQkq", "-", "0", "1").unwrap();
        assert_eq!(state.to_board_string(), "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR");
    }

    #[test]
    fn parse_fen_test2() {
        let state =
            FenState::from("r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1", "w", "-", "-", "0", "10").unwrap();

        assert_eq!(state.to_board_string(), "r..q.rk.pp..bppp..n.pn....bp........P.....NP.N..PPQ..PPPR.B..RK.");
    }

    #[test]
    fn parse_fen_test3() {
        let state =
            FenState::from("r1bqk2r/pp1n1ppp/2pbpn2/8/3P4/2N1BN2/PPP2PPP/R2QKB1R", "w", "KQkq", "-", "6", "7").unwrap();

        assert_eq!(state.to_board_string(), "r.bqk..rpp.n.ppp..pbpn.............P......N.BN..PPP..PPPR..QKB.R");
    }
}
