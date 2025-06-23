use crate::core::position::Position;
use crate::core::types::{Color, Piece, PieceType};

#[rustfmt::skip]
pub const PST_PAWN_START: [i32;64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
    50,  50,  50,  50,  50,  50,  50,  50,
    10,  10,  20,  30,  30,  20,  10,  10,
     5,   5,  10,  25,  25,  10,   5,   5,
     0,   0,   0,  20,  20,   0,   0,   0,
     5,  -5, -10,   0,   0, -10,  -5,   5,
     5,  10,  10, -20, -20,  10,  10,   5,
     0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
pub const PST_PAWN_END: [i32;64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
    80,  80,  80,  80,  80,  80,  80,  80,
    50,  50,  50,  50,  50,  50,  50,  50,
    30,  30,  30,  30,  30,  30,  30,  30,
    20,  20,  20,  20,  20,  20,  20,  20,
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
     0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
pub const PST_KNIGHT: [i32;64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
pub const PST_BISHOP: [i32;64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
pub const PST_ROOK: [i32;64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
pub const PST_QUEEN: [i32;64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
pub const PST_KING_START: [i32;64] = [
    -80, -70, -70, -70, -70, -70, -70, -80,
    -60, -60, -60, -60, -60, -60, -60, -60,
    -40, -50, -50, -60, -60, -50, -50, -40,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,  -5,  -5,  -5,  -5,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20
];

#[rustfmt::skip]
pub const PST_KING_END: [i32;64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
     -5,   0,   5,   5,   5,   5,   0,  -5,
    -10,  -5,  20,  30,  30,  20,  -5, -10,
    -15, -10,  35,  45,  45,  35, -10, -15,
    -20, -15,  30,  40,  40,  30, -15, -20,
    -25, -20,  20,  25,  25,  20, -20, -25,
    -30, -25,   0,   0,   0,   0, -25, -30,
    -50, -30, -30, -30, -30, -30, -30, -50
];

const fn flip_table(input: &[i32; 64]) -> [i32; 64] {
    let mut output = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        let mirror = 56 ^ sq;
        output[sq] = input[mirror];
        sq += 1
    }
    output
}

pub type PieceSquareTable = [[i32; 64]; 2];

pub const KNIGHT_TABLES: PieceSquareTable = [PST_KNIGHT, flip_table(&PST_KNIGHT)];
pub const BISHOP_TABLES: PieceSquareTable = [PST_BISHOP, flip_table(&PST_BISHOP)];
pub const ROOK_TABLES: PieceSquareTable = [PST_ROOK, flip_table(&PST_ROOK)];
pub const QUEEN_TABLES: PieceSquareTable = [PST_QUEEN, flip_table(&PST_QUEEN)];

pub const PAWN_START_TABLES: PieceSquareTable = [PST_PAWN_START, flip_table(&PST_PAWN_START)];
pub const PAWN_END_TABLES: PieceSquareTable = [PST_PAWN_END, flip_table(&PST_PAWN_END)];
pub const KING_START_TABLES: PieceSquareTable = [PST_KING_START, flip_table(&PST_KING_START)];
pub const KING_END_TABLES: PieceSquareTable = [PST_KING_END, flip_table(&PST_KING_END)];

pub fn evaluate_table(
    pos: &Position,
    piece_square_table: &PieceSquareTable,
    piece_type: PieceType,
    color: Color,
) -> i32 {
    let piece = Piece::get_piece(color, piece_type);
    let bitboard = pos.bitboards[piece.as_usize()];

    let mut score = 0;
    for sq in bitboard.iter() {
        score += piece_square_table[color.as_usize()][sq.as_u8() as usize];
    }

    score
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_table_helper(table1: &[i32; 64], table2: &[i32; 64]) {
        for f in 0..8 {
            for r in 0..8 {
                let r1 = r;
                let r2 = 7 - r;
                assert_eq!(table1[f + r1 * 8 as usize], table2[f + r2 * 8 as usize]);
            }
        }
    }

    #[test]
    fn test_table_mirrored() {
        test_table_helper(&PAWN_START_TABLES[0], &PAWN_START_TABLES[1]);
        test_table_helper(&PAWN_END_TABLES[0], &PAWN_END_TABLES[1]);
        test_table_helper(&KNIGHT_TABLES[0], &KNIGHT_TABLES[1]);
        test_table_helper(&BISHOP_TABLES[0], &BISHOP_TABLES[1]);
        test_table_helper(&ROOK_TABLES[0], &ROOK_TABLES[1]);
        test_table_helper(&QUEEN_TABLES[0], &QUEEN_TABLES[1]);
        test_table_helper(&KING_START_TABLES[0], &KING_START_TABLES[1]);
        test_table_helper(&KING_END_TABLES[0], &KING_END_TABLES[1]);
    }

    #[test]
    fn test_evaluate_table() {
        let pos = Position::new();
        let score = evaluate_table(&pos, &KNIGHT_TABLES, PieceType::KNIGHT, Color::WHITE);
        assert_eq!(score, -80, "Initial position should have a score of -40 for each knights");
        let score = evaluate_table(&pos, &KNIGHT_TABLES, PieceType::KNIGHT, Color::BLACK);
        assert_eq!(score, -80, "Initial position should have a score of -40 for each knights");
    }
}
