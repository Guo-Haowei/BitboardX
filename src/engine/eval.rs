use crate::core::{position::Position, types::*};

const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    305,   // Knight
    305,   // Bishop
    490,   // Rook
    1000,  // Queen
    40000, // King
];

fn evaluate_material(pos: &Position, color: Color) -> i32 {
    // exclude king from material evaluation
    let mut score = 0;
    for i in 0..PieceType::KING.0 {
        let piece = Piece::get_piece(color, PieceType(i));
        score += PIECE_VALUES[i as usize] * pos.bitboards[piece.as_usize()].count() as i32;
    }

    score
}

fn evaluate_pst(pos: &Position) -> i32 {
    let mut scores = [0; 2];

    for i in 0..Piece::COUNT {
        let piece = Piece::new(i as u8);
        let color = piece.color();
        let bitboard = pos.bitboards[piece.as_usize()];
        for sq in bitboard.iter() {
            scores[color.as_usize()] += PST_TABLE[piece.as_usize()][sq.as_u8() as usize];
        }
    }

    scores[0] - scores[1]
}

// @TODO: Implement pawn structure, mobility, rook activity, bishop pair, etc.
fn evaluate_king_safety(_pos: &Position) -> i32 {
    // Placeholder for king safety evaluation
    0
}

// for simplicity, we use mid-game piece value table
fn get_piece_value(piece_type: PieceType) -> i32 {
    assert!(piece_type != PieceType::NONE, "Piece must not be NONE");
    PIECE_VALUES[piece_type.as_u8() as usize]
}

pub fn evaluate(pos: &Position) -> i32 {
    debug_assert!(pos.side_to_move == Color::WHITE || pos.side_to_move == Color::BLACK);
    let mut score = 0;

    score += evaluate_material(pos, Color::WHITE) - evaluate_material(pos, Color::BLACK);
    score += evaluate_pst(pos);
    score += evaluate_king_safety(pos);

    if pos.side_to_move == Color::WHITE { score } else { -score }
}

pub fn move_score_guess(pos: &Position, mv: Move) -> i32 {
    let move_type = mv.get_type();
    let src_sq = mv.src_sq();
    let dst_sq = mv.dst_sq();
    let color = pos.side_to_move;
    let opponent = color.opponent();
    let src_piece = pos.get_piece_at(src_sq);
    let captured_piece = if move_type == MoveType::EnPassant {
        Piece::get_piece(opponent, PieceType::PAWN)
    } else {
        pos.get_piece_at(dst_sq)
    };
    let src_piece_value = get_piece_value(src_piece.get_type());

    let mut guess = 0;

    // prioritize capture high value piece with low value piece
    if captured_piece != Piece::NONE {
        let captured_piece_value = get_piece_value(captured_piece.get_type());
        guess = 10 * captured_piece_value - src_piece_value;
    }

    // promote a pawn is also a good move
    if move_type == MoveType::Promotion {
        let promo_piece = mv.get_promotion().unwrap();
        guess += get_piece_value(promo_piece);
    }

    // penalize moving a piece to a square that is attacked by an opponent piece
    if pos.attack_mask[opponent.as_usize()].test(dst_sq.as_u8()) {
        guess = -src_piece_value;
    }

    guess
}

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

#[rustfmt::skip]
const PST_WHITE_PAWN: [i32; 64] =
   [ 0,  0,  0,  0,  0,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  3,  5,  5,  3,  0, -5,
    -5,  0,  5, 10, 10,  5,  0, -5,
    -5,  0,  3,  5,  5,  3,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  0,  0,  0,  0,  0 ];

#[rustfmt::skip]
const PST_WHITE_KNIGHT: [i32; 64] =
   [ -31, -29, -27, -25, -25, -27, -29, -31,
      -9,  -6,  -2,   0,   0,  -2,  -6,  -9,
      -7,  -2,  19,  19,  19,  19,  -2,  -7,
      -5,  10,  23,  28,  28,  23,  10,  -5,
      -5,  12,  25,  32,  32,  25,  12,  -5,
      -7,  10,  23,  29,  29,  23,  10,  -7,
      -9,   4,  14,  20,  20,  14,   4,  -9,
     -41, -29, -27, -15, -15, -27, -29, -41 ];

#[rustfmt::skip]
const PST_WHITE_BISHOP: [i32; 64] =
    [-15, -15, -15, -15, -15, -15, -15, -15,
      0,   4,   4,   4,   4,   4,   4,   0,
      0,   4,   8,   8,   8,   8,   4,   0,
      0,   4,   8,  12,  12,   8,   4,   0,
      0,   4,   8,  12,  12,   8,   4,   0,
      0,   4,   8,   8,   8,   8,   4,   0,
      0,   4,   4,   4,   4,   4,   4,   0,
      0,   0,   0,   0,   0,   0,   0,   0 ];

// Rook will be evaluated using open files and ranks, so we use a placeholder table
#[rustfmt::skip]
const PST_WHITE_ROOK: [i32; 64] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26 ];

#[rustfmt::skip]
const PST_WHITE_QUEEN: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   4,   4,   4,   4,   0,   0,
      0,   4,   4,   6,   6,   4,   4,   0,
      0,   4,   6,   8,   8,   6,   4,   0,
      0,   4,   6,   8,   8,   6,   4,   0,
      0,   4,   4,   6,   6,   4,   4,   0,
      0,   0,   4,   4,   4,   4,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0 ];

#[rustfmt::skip]
const PST_WHITE_KING: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14 ];

const PST_BLACK_PAWN: [i32; 64] = flip_table(&PST_WHITE_PAWN);
const PST_BLACK_KNIGHT: [i32; 64] = flip_table(&PST_WHITE_KNIGHT);
const PST_BLACK_BISHOP: [i32; 64] = flip_table(&PST_WHITE_BISHOP);
const PST_BLACK_ROOK: [i32; 64] = flip_table(&PST_WHITE_ROOK);
const PST_BLACK_QUEEN: [i32; 64] = flip_table(&PST_WHITE_QUEEN);
const PST_BLACK_KING: [i32; 64] = flip_table(&PST_WHITE_KING);

#[allow(dead_code)]
const PST_TABLE: [[i32; 64]; Piece::COUNT] = [
    PST_WHITE_PAWN,
    PST_WHITE_KNIGHT,
    PST_WHITE_BISHOP,
    PST_WHITE_ROOK,
    PST_WHITE_QUEEN,
    PST_WHITE_KING,
    PST_BLACK_PAWN,
    PST_BLACK_KNIGHT,
    PST_BLACK_BISHOP,
    PST_BLACK_ROOK,
    PST_BLACK_QUEEN,
    PST_BLACK_KING,
];

#[cfg(test)]
mod test {
    use super::*;

    fn test_table(table1: &[i32; 64], table2: &[i32; 64]) {
        for f in 0..8 {
            for r in 0..8 {
                let r1 = r;
                let r2 = 7 - r;
                assert_eq!(table1[f + r1 * 8 as usize], table2[f + r2 * 8 as usize]);
            }
        }
    }

    #[test]
    fn test_pst() {
        test_table(&PST_BLACK_KNIGHT, &PST_WHITE_KNIGHT);
        test_table(&PST_WHITE_PAWN, &PST_BLACK_PAWN);
        test_table(&PST_WHITE_BISHOP, &PST_BLACK_BISHOP);
        test_table(&PST_WHITE_ROOK, &PST_BLACK_ROOK);
        test_table(&PST_WHITE_QUEEN, &PST_BLACK_QUEEN);
        test_table(&PST_WHITE_KING, &PST_BLACK_KING);
    }
}
