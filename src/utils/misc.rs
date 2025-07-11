use crate::core::position::Position;
use crate::core::types::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn random() -> f32 {
    use rand::Rng;
    let mut rng = rand::rng();
    let num = rng.random();
    debug_assert!(num <= 1.0, "Random number out of bounds: {}", num);
    num
}

#[cfg(target_arch = "wasm32")]
pub fn random() -> f32 {
    use web_sys::js_sys::Math;
    Math::random() as f32
}

fn parse_move_impl(input: &str) -> Option<(Square, Square, Option<PieceType>)> {
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

            if len == 4 {
                return Some((src, dst, None));
            }

            let promotion = match input.chars().nth(4)? {
                'q' | 'Q' => Some(PieceType::QUEEN),
                'r' | 'R' => Some(PieceType::ROOK),
                'b' | 'B' => Some(PieceType::BISHOP),
                'n' | 'N' => Some(PieceType::KNIGHT),
                _ => return None,
            };

            Some((src, dst, promotion))
        }
        _ => return None,
    }
}

pub fn parse_move(input: &str) -> Option<Move> {
    if let Some((src, dst, promotion)) = parse_move_impl(input) {
        return match promotion {
            Some(_) => Some(Move::new(src, dst, MoveType::Promotion, promotion)),
            None => Some(Move::new(src, dst, MoveType::Normal, None)),
        };
    }

    None
}

pub fn board_string(pos: &Position) -> String {
    let mut vec = vec![b'.'; 64];

    for i in Piece::W_PAWN.as_usize()..=Piece::B_KING.as_usize() {
        let piece = Piece::new(i as u8);
        for sq in pos.bitboards[piece.as_usize()].iter() {
            let idx = sq.as_usize();
            vec[idx] = piece.to_char() as u8;
        }
    }

    String::from_utf8(vec).unwrap()
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
        assert_eq!(parse_move_impl("e2e4"), Some((Square::E2, Square::E4, None)));
        assert_eq!(parse_move_impl("a7a8"), Some((Square::A7, Square::A8, None)));
        assert_eq!(parse_move_impl("h1h2"), Some((Square::H1, Square::H2, None)));
        assert_eq!(parse_move_impl("d4d5"), Some((Square::D4, Square::D5, None)));
        assert_eq!(
            parse_move_impl("d7d8q"),
            Some((Square::D7, Square::D8, Some(PieceType::QUEEN)))
        );
        assert_eq!(parse_move_impl("g2g1r"), Some((Square::G2, Square::G1, Some(PieceType::ROOK))));
        assert_eq!(parse_move_impl("z1z2"), None);
        assert_eq!(parse_move_impl("e9e4"), None);
        assert_eq!(parse_move_impl("e2e"), None);
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
