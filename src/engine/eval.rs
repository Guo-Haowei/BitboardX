use super::piece_square_table::*;
use crate::core::{position::Position, types::*};

const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    305,   // Knight
    305,   // Bishop
    490,   // Rook
    1000,  // Queen
    40000, // King
];

struct EvaluationData {
    material_score: i32,
    mop_up_score: i32, // score for endgame material
    piece_square_score: i32,
    pawn_score: i32,
    pawn_shield_score: i32,
}

impl EvaluationData {
    pub fn new() -> Self {
        EvaluationData {
            material_score: 0,
            mop_up_score: 0,
            piece_square_score: 0,
            pawn_score: 0,
            pawn_shield_score: 0,
        }
    }

    pub fn sum(&self) -> i32 {
        self.material_score
            + self.mop_up_score
            + self.piece_square_score
            + self.pawn_score
            + self.pawn_shield_score
    }
}

struct MaterialInfo {
    pub material_score: i32,
    pub _num_pawns: i32,
    pub _num_knights: i32,
    pub _num_bishops: i32,
    pub _num_queens: i32,
    pub _num_rooks: i32,
    pub _my_pawns: BitBoard,
    pub _enemy_pawns: BitBoard,
    pub endgame_t: f32, // Transition from midgame to endgame (0->1)
}

impl MaterialInfo {
    fn new(
        num_pawns: i32,
        num_knights: i32,
        num_bishops: i32,
        num_queens: i32,
        num_rooks: i32,
        my_pawns: BitBoard,
        enemy_pawns: BitBoard,
    ) -> Self {
        let mut material_score = 0;
        material_score += num_pawns * PAWN_VALUE;
        material_score += num_knights * KNIGHT_VALUE;
        material_score += num_bishops * BISHOP_VALUE;
        material_score += num_rooks * ROOK_VALUE;
        material_score += num_queens * QUEEN_VALUE;

        // Endgame Transition (0->1)
        const QUEEN_ENDGAME_WEIGHT: i32 = 45;
        const ROOK_ENDGAME_WEIGHT: i32 = 20;
        const BISHOP_ENDGAME_WEIGHT: i32 = 10;
        const KNIGHT_ENDGAME_WEIGHT: i32 = 10;

        const ENDGAME_START_WEIGHT: i32 = 2 * ROOK_ENDGAME_WEIGHT
            + 2 * BISHOP_ENDGAME_WEIGHT
            + 2 * KNIGHT_ENDGAME_WEIGHT
            + QUEEN_ENDGAME_WEIGHT;
        let endgame_weight_sum = num_queens * QUEEN_ENDGAME_WEIGHT
            + num_rooks * ROOK_ENDGAME_WEIGHT
            + num_bishops * BISHOP_ENDGAME_WEIGHT
            + num_knights * KNIGHT_ENDGAME_WEIGHT;

        let endgame_t = 1.0 - (endgame_weight_sum as f32 / ENDGAME_START_WEIGHT as f32).min(1.0);

        MaterialInfo {
            material_score,
            _num_pawns: num_pawns,
            _num_knights: num_knights,
            _num_bishops: num_bishops,
            _num_queens: num_queens,
            _num_rooks: num_rooks,
            _my_pawns: my_pawns,
            _enemy_pawns: enemy_pawns,
            endgame_t,
        }
    }
}

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 320;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// const PASSED_PAWN_BONUSES: [i32; 7] = [0, 120, 80, 50, 30, 15, 15];
// const ISOLATED_PAWN_PENALTY_BY_COUNT: [i32; 9] = [0, -10, -25, -50, -75, -75, -75, -75, -75];
// const KING_PAWN_SHIELD_SCORES: [i32; 6] = [4, 7, 4, 3, 6, 3];

// const ENDGAME_MATERIAL_START: i32 = ROOK_VALUE * 2 + BISHOP_VALUE + KNIGHT_VALUE;

pub struct Evaluation {
    white_score: EvaluationData,
    black_score: EvaluationData,
}

impl Evaluation {
    pub fn new() -> Self {
        Evaluation { white_score: EvaluationData::new(), black_score: EvaluationData::new() }
    }

    pub fn evaluate_position(&mut self, pos: &Position) -> i32 {
        let white_material = self.get_material_info(pos, Color::WHITE);
        let black_material = self.get_material_info(pos, Color::BLACK);

        // Score based on material left on the board
        self.white_score.material_score = white_material.material_score;
        self.black_score.material_score = black_material.material_score;

        // Score based on piece-square tables
        self.white_score.piece_square_score =
            self.evaluate_piece_square_table(pos, Color::WHITE, white_material.endgame_t);
        self.black_score.piece_square_score =
            self.evaluate_piece_square_table(pos, Color::BLACK, black_material.endgame_t);

        // Push the king to edge of the board in endgame

        // Evaluate pawns (passed, isolated, sheild)

        let perspective = if pos.side_to_move == Color::WHITE { 1 } else { -1 };
        let score = self.white_score.sum() - self.black_score.sum();

        // eprintln!(
        //     "Evaluation: {} (White: {}, Black: {})",
        //     score,
        //     self.white_score.sum(),
        //     self.black_score.sum()
        // );

        score * perspective
    }

    fn get_material_info(&self, pos: &Position, color: Color) -> MaterialInfo {
        let pawn = Piece::get_piece(color, PieceType::PAWN);
        let knight = Piece::get_piece(color, PieceType::KNIGHT);
        let bishop = Piece::get_piece(color, PieceType::BISHOP);
        let rook = Piece::get_piece(color, PieceType::ROOK);
        let queen = Piece::get_piece(color, PieceType::QUEEN);

        let my_pawns = pos.bitboards[pawn.as_usize()];
        let enemy_pawns = Piece::get_piece(color.opponent(), PieceType::PAWN);
        let enemy_pawns = pos.bitboards[enemy_pawns.as_usize()];

        let num_pawns = my_pawns.count() as i32;
        let num_knights = pos.bitboards[knight.as_usize()].count() as i32;
        let num_bishops = pos.bitboards[bishop.as_usize()].count() as i32;
        let num_rooks = pos.bitboards[rook.as_usize()].count() as i32;
        let num_queens = pos.bitboards[queen.as_usize()].count() as i32;

        MaterialInfo::new(
            num_pawns,
            num_knights,
            num_bishops,
            num_queens,
            num_rooks,
            my_pawns,
            enemy_pawns,
        )
    }

    fn evaluate_piece_square_table(&self, pos: &Position, color: Color, endgame_t: f32) -> i32 {
        let mut value = 0;
        value += evaluate_table(pos, &KNIGHT_TABLES, PieceType::KNIGHT, color);
        value += evaluate_table(pos, &BISHOP_TABLES, PieceType::BISHOP, color);
        value += evaluate_table(pos, &ROOK_TABLES, PieceType::ROOK, color);
        value += evaluate_table(pos, &QUEEN_TABLES, PieceType::QUEEN, color);

        let pawn_early = evaluate_table(pos, &PAWN_START_TABLES, PieceType::PAWN, color);
        let pawn_late = evaluate_table(pos, &PAWN_END_TABLES, PieceType::PAWN, color);
        value += (pawn_early as f32 * (1.0 - endgame_t)) as i32;
        value += (pawn_late as f32 * endgame_t) as i32;

        let king_early = evaluate_table(pos, &KING_START_TABLES, PieceType::KING, color);
        let king_late = evaluate_table(pos, &KING_END_TABLES, PieceType::KING, color);
        value += (king_early as f32 * (1.0 - endgame_t)) as i32;
        value += (king_late as f32 * endgame_t) as i32;

        value
    }
}

// for simplicity, we use mid-game piece value table
fn get_piece_value(piece_type: PieceType) -> i32 {
    assert!(piece_type != PieceType::NONE, "Piece must not be NONE");
    PIECE_VALUES[piece_type.as_u8() as usize]
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
