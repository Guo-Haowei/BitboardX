use super::position::Position;
use crate::core::types::*;

pub fn parse_move(input: &str) -> Option<(Square, Square, Option<PieceType>)> {
    let len = input.len();
    match len {
        4 | 5 => {
            let f1 = input.chars().nth(0)? as u8 - b'a';
            let r1 = input.chars().nth(1)? as u8 - b'1';
            let f2 = input.chars().nth(2)? as u8 - b'a';
            let r2 = input.chars().nth(3)? as u8 - b'1';
            if f1 > 7 || r1 > 7 || f2 > 7 || r2 > 7 {
                return None;
            }

            let src = Square::make(File(f1), Rank(r1));
            let dst = Square::make(File(f2), Rank(r2));

            if len == 5 {
                let promotion = match input.chars().nth(4)? {
                    'q' | 'Q' => Some(PieceType::QUEEN),
                    'r' | 'R' => Some(PieceType::ROOK),
                    'b' | 'B' => Some(PieceType::BISHOP),
                    'n' | 'N' => Some(PieceType::KNIGHT),
                    _ => return None,
                };
                return Some((src, dst, promotion));
            }

            Some((src, dst, None))
        }
        _ => return None,
    }
}

pub fn debug_string(pos: &Position) -> String {
    let mut s = String::new();
    for rank in (0..8).rev() {
        s.push((rank as u8 + b'1') as char);
        s.push(' ');
        for file in 0..8 {
            let sq = rank * 8 + file;
            let piece_char = if pos.bitboards[Piece::W_PAWN.as_usize()].test(sq) {
                '♙'
            } else if pos.bitboards[Piece::W_KNIGHT.as_usize()].test(sq) {
                '♘'
            } else if pos.bitboards[Piece::W_BISHOP.as_usize()].test(sq) {
                '♗'
            } else if pos.bitboards[Piece::W_ROOK.as_usize()].test(sq) {
                '♖'
            } else if pos.bitboards[Piece::W_QUEEN.as_usize()].test(sq) {
                '♕'
            } else if pos.bitboards[Piece::W_KING.as_usize()].test(sq) {
                '♔'
            } else if pos.bitboards[Piece::B_PAWN.as_usize()].test(sq) {
                '♟'
            } else if pos.bitboards[Piece::B_KNIGHT.as_usize()].test(sq) {
                '♞'
            } else if pos.bitboards[Piece::B_BISHOP.as_usize()].test(sq) {
                '♝'
            } else if pos.bitboards[Piece::B_ROOK.as_usize()].test(sq) {
                '♜'
            } else if pos.bitboards[Piece::B_QUEEN.as_usize()].test(sq) {
                '♛'
            } else if pos.bitboards[Piece::B_KING.as_usize()].test(sq) {
                '♚'
            } else {
                '.'
            };

            if piece_char == '.' {
                s.push('・');
            } else {
                s.push(piece_char);
                if !cfg!(target_arch = "wasm32") {
                    s.push(' ');
                }
            }
        }
        s.push('\n');
    }
    s.push_str("  ａｂｃｄｅｆｇｈ\n");

    s.push_str(format!("\nFen: {}\n", pos.fen()).as_str());

    s
}

pub fn min_max<T: PartialOrd + Copy>(a: T, b: T) -> (T, T) {
    if a < b { (a, b) } else { (b, a) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_move() {
        assert_eq!(parse_move("e2e4"), Some((Square::E2, Square::E4, None)));
        assert_eq!(parse_move("a7a8"), Some((Square::A7, Square::A8, None)));
        assert_eq!(parse_move("h1h2"), Some((Square::H1, Square::H2, None)));
        assert_eq!(parse_move("d4d5"), Some((Square::D4, Square::D5, None)));
        assert_eq!(parse_move("d7d8q"), Some((Square::D7, Square::D8, Some(PieceType::QUEEN))));
        assert_eq!(parse_move("z1z2"), None);
        assert_eq!(parse_move("e9e4"), None);
        assert_eq!(parse_move("e2e"), None);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(min_max(3, 5), (3, 5));
        assert_eq!(min_max(5, 3), (3, 5));
        assert_eq!(min_max(7.2, 4.8), (4.8, 7.2));
        assert_eq!(min_max(-1, 1), (-1, 1));
        assert_eq!(min_max(0, 0), (0, 0));
    }
}
