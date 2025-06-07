use crate::types::*;

pub struct Board {
    bitboards: [u64; Piece::Count as usize],
    // occupancies: [u64; 3],
    // side_to_move: Color,
}

impl Board {
    pub fn new() -> Self {
        let mut ret = Self {
            bitboards: [0; Piece::Count as usize],
            // occupancies: [0; 3],
            // side_to_move: Color::White,
        };
        ret.bitboards[Piece::WhitePawn as usize] = 0x000000000000FF00;
        ret.bitboards[Piece::BlackPawn as usize] = 0x00FF000000000000;
        ret.bitboards[Piece::WhiteKing as usize] = 0x0000000000000010;
        ret.bitboards[Piece::BlackKing as usize] = 0x1000000000000000;
        ret
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
