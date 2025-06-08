use crate::types::*;

pub struct Board {
    bitboards: [u64; Piece::Count as usize],
    // occupancies: [u64; 3],
    // side_to_move: Color,
}

impl Board {
    pub fn new() -> Self {
        Self {
            bitboards: [0; Piece::Count as usize],
            // occupancies: [0; 3],
            // side_to_move: Color::White,
        }
    }

    pub fn parse_fen(&mut self, fen: &str) -> Result<(), String> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields".to_string());
        }

        for (rank, rank_str) in parts[0].split('/').enumerate() {
            let mut file = 0;
            for c in rank_str.chars() {
                let sq = ((rank as u8) << 3) + file as u8;
                let mut inc = 1;
                match c {
                    'p' => self.bitboards[Piece::BlackPawn as usize] |= 1 << sq,
                    'r' => self.bitboards[Piece::BlackRook as usize] |= 1 << sq,
                    'n' => self.bitboards[Piece::BlackKnight as usize] |= 1 << sq,
                    'b' => self.bitboards[Piece::BlackBishop as usize] |= 1 << sq,
                    'q' => self.bitboards[Piece::BlackQueen as usize] |= 1 << sq,
                    'k' => self.bitboards[Piece::BlackKing as usize] |= 1 << sq,
                    'P' => self.bitboards[Piece::WhitePawn as usize] |= 1 << sq,
                    'R' => self.bitboards[Piece::WhiteRook as usize] |= 1 << sq,
                    'N' => self.bitboards[Piece::WhiteKnight as usize] |= 1 << sq,
                    'B' => self.bitboards[Piece::WhiteBishop as usize] |= 1 << sq,
                    'Q' => self.bitboards[Piece::WhiteQueen as usize] |= 1 << sq,
                    'K' => self.bitboards[Piece::WhiteKing as usize] |= 1 << sq,
                    '1'..='8' => inc = c.to_digit(10).unwrap(),
                    _ => return Err(format!("Invalid character '{}' in board layout", c))
                }

                file += inc as usize;
            }
            if file != 8 {
                return Err("Invalid board layout in FEN".to_string());
            }
        }
        println!();

        Ok(())
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();

        for rank in (0..8).rev() {
            result.push(((rank + 1) as u8 + b'0') as char);
            result.push(' ');
            let rank_str = self.rank_string(rank);
            result.push_str(&rank_str);
            result.push('\n');
        }
        result.push_str("  a b c d e f g h");

        result
    }

    fn rank_string(&self, rank: u8) -> String {
        assert!(rank < Rank::Count as u8);

        let mut result = String::new();
        for file in 0..8 {
            let sq = rank * 8 + file;
            let mask = 1u64 << sq;
            let mut piece_char = '.';
            for i in 0..self.bitboards.len() {
                if (self.bitboards[i] & mask) != 0 {
                    let piece : Piece = unsafe { std::mem::transmute(i as u8) };
                    piece_char = match piece {
                        Piece::WhitePawn => 'P',
                        Piece::WhiteKnight => 'N',
                        Piece::WhiteBishop => 'B',
                        Piece::WhiteRook => 'R',
                        Piece::WhiteQueen => 'Q',
                        Piece::WhiteKing => 'K',
                        Piece::BlackPawn => 'p',
                        Piece::BlackKnight => 'n',
                        Piece::BlackBishop => 'b',
                        Piece::BlackRook => 'r',
                        Piece::BlackQueen => 'q',
                        Piece::BlackKing => 'k',
                        Piece::Count => '.',
                    };
                    break;
                }
            }
            result.push(piece_char);
            result.push(' ');
        }
        result
    }
}
