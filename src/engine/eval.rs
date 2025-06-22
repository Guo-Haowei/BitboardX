use super::piece_square_table::*;
use crate::core::{position::Position, types::*};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 320;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

const PIECE_VALUES: [i32; 6] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    40000, // King
];

// for simplicity, we use mid-game piece value table
pub fn get_piece_value(piece_type: PieceType) -> i32 {
    assert!(piece_type != PieceType::NONE, "Piece must not be NONE");
    PIECE_VALUES[piece_type.as_u8() as usize]
}

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
    pub color: Color,
    pub material_score: i32,
    pub _num_pawns: i32,
    pub _num_knights: i32,
    pub _num_bishops: i32,
    pub _num_queens: i32,
    pub _num_rooks: i32,
    pub my_pawns: BitBoard,
    pub enemy_pawns: BitBoard,
    pub endgame_t: f32, // Transition from midgame to endgame (0->1)
}

impl MaterialInfo {
    fn new(
        color: Color,
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
            color,
            material_score,
            _num_pawns: num_pawns,
            _num_knights: num_knights,
            _num_bishops: num_bishops,
            _num_queens: num_queens,
            _num_rooks: num_rooks,
            my_pawns,
            enemy_pawns,
            endgame_t,
        }
    }
}

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
        let white_material = Self::get_material_info(pos, Color::WHITE);
        let black_material = Self::get_material_info(pos, Color::BLACK);

        // Score based on material left on the board
        self.white_score.material_score = white_material.material_score;
        self.black_score.material_score = black_material.material_score;

        // Score based on piece-square tables
        self.white_score.piece_square_score =
            self.evaluate_piece_square_table(pos, Color::WHITE, white_material.endgame_t);
        self.black_score.piece_square_score =
            self.evaluate_piece_square_table(pos, Color::BLACK, black_material.endgame_t);

        // Evaluate pawns (passed, isolated, sheild)
        self.white_score.pawn_score = self.evaluate_pawns(&white_material);
        self.black_score.pawn_score = self.evaluate_pawns(&black_material);

        // Push the king to edge of the board in endgame (for endgame checkmate)

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

    fn get_material_info(pos: &Position, color: Color) -> MaterialInfo {
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
            color,
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

    fn evaluate_pawns(&self, material: &MaterialInfo) -> i32 {
        let mut score = 0;
        score += Self::evaluate_passed_pawns(material);
        score += Self::evaluate_isolated_pawns(material);
        score
    }

    fn evaluate_passed_pawns(material: &MaterialInfo) -> i32 {
        const PASSED_PAWN_BONUSES: [i32; 7] = [0, 120, 80, 50, 30, 15, 15];

        let mut score = 0;

        for sq in material.my_pawns.iter() {
            let mask = PASSED_PAWN_MASKS[material.color.as_usize()][sq.as_u8() as usize];
            if (material.enemy_pawns & mask).none() {
                let (_, rank) = sq.file_rank();
                debug_assert!(rank.0 < 7);
                let idx = if material.color == Color::WHITE { rank.0 } else { 7 - rank.0 };
                score += PASSED_PAWN_BONUSES[idx as usize];
            }
        }

        score
    }

    fn evaluate_isolated_pawns(material: &MaterialInfo) -> i32 {
        const ISOLATED_PAWN_PENALTY_BY_COUNT: [i32; 9] =
            [0, -10, -25, -50, -75, -75, -75, -75, -75];

        let mut isolated_count = 0;
        for sq in material.my_pawns.iter() {
            let (file, _) = sq.file_rank();
            let mask = ISOLATED_PAWN_MASKS[file.0 as usize];
            if (material.my_pawns & mask).none() {
                isolated_count += 1;
            }
        }

        ISOLATED_PAWN_PENALTY_BY_COUNT[isolated_count as usize]
    }
}

const FILE_MASKS: [u64; 8] = [
    0x0101010101010101, // A file
    0x0202020202020202, // B file
    0x0404040404040404, // C file
    0x0808080808080808, // D file
    0x1010101010101010, // E file
    0x2020202020202020, // F file
    0x4040404040404040, // G file
    0x8080808080808080, // H file
];

const RANK_MASKS: [u64; 8] = [
    0x00000000000000FF, // Rank 1
    0x000000000000FF00, // Rank 2
    0x0000000000FF0000, // Rank 3
    0x00000000FF000000, // Rank 4
    0x000000FF00000000, // Rank 5
    0x0000FF0000000000, // Rank 6
    0x00FF000000000000, // Rank 7
    0xFF00000000000000, // Rank 8
];

const fn adjacent_files_mask(file: File) -> BitBoard {
    let mut mask = 0u64;
    let f = file.0 as i8;

    if f > 0 {
        mask |= FILE_MASKS[(f - 1) as usize];
    }
    if f < 7 {
        mask |= FILE_MASKS[(f + 1) as usize];
    }

    BitBoard::from(mask)
}

const fn passed_pawn_mask<const IS_WHITE: bool>(square: Square) -> BitBoard {
    let mut mask = !0u64;
    let (file, rank) = square.file_rank();

    let mut f = 0i8;
    while f < 8 {
        let diff = file.0 as i8 - f;
        let diff = diff.abs();
        if diff <= 1 {
            mask &= !FILE_MASKS[f as usize];
        }

        f += 1;
    }

    let mut r = rank.0 as i8;
    if IS_WHITE {
        while r >= 0 {
            mask |= RANK_MASKS[r as usize];
            r -= 1;
        }
    } else {
        while r < 8 {
            mask |= RANK_MASKS[r as usize];
            r += 1;
        }
    }

    BitBoard::from(!mask)
}

const ISOLATED_PAWN_MASKS: [BitBoard; 8] = [
    adjacent_files_mask(File::A),
    adjacent_files_mask(File::B),
    adjacent_files_mask(File::C),
    adjacent_files_mask(File::D),
    adjacent_files_mask(File::E),
    adjacent_files_mask(File::F),
    adjacent_files_mask(File::G),
    adjacent_files_mask(File::H),
];

const fn passed_pawn_mask_all<const IS_WHITE: bool>() -> [BitBoard; 64] {
    let mut masks = [BitBoard::new(); 64];
    let mut sq = 0u8;
    while sq < 64 {
        masks[sq as usize] = passed_pawn_mask::<IS_WHITE>(Square::new(sq));
        sq += 1;
    }
    masks
}

const PASSED_PAWN_MASKS: [[BitBoard; 64]; 2] = [
    passed_pawn_mask_all::<true>(),  // White
    passed_pawn_mask_all::<false>(), // White
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacent_files_mask() {
        let mask = adjacent_files_mask(File::A);
        assert_eq!(mask.get(), BitBoard::MASK_B);
        let mask = adjacent_files_mask(File::B);
        assert_eq!(mask.get(), BitBoard::MASK_A | BitBoard::MASK_C);
        let mask = adjacent_files_mask(File::F);
        assert_eq!(mask.get(), BitBoard::MASK_E | BitBoard::MASK_G);
        let mask = adjacent_files_mask(File::H);
        assert_eq!(mask.get(), BitBoard::MASK_G);
    }

    #[test]
    fn test_passed_pawn_mask() {
        const F4_MASK: BitBoard = passed_pawn_mask::<true>(Square::F4);

        assert_eq!(
            F4_MASK.to_string(),
            r#"0 0 0 0 1 1 1 0
0 0 0 0 1 1 1 0
0 0 0 0 1 1 1 0
0 0 0 0 1 1 1 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0
"#
        );
    }

    #[test]
    fn test_passed_pawns() {
        let fen = "rnbqkbnr/3pp2p/8/6P1/6p1/1P3p2/P1PP4/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        let white_material = Evaluation::get_material_info(&pos, Color::WHITE);
        let white_passed_pawns_score = Evaluation::evaluate_passed_pawns(&white_material);
        assert_eq!(white_passed_pawns_score, 200);

        let black_material = Evaluation::get_material_info(&pos, Color::BLACK);
        let black_passed_pawns_score = Evaluation::evaluate_passed_pawns(&black_material);
        assert_eq!(black_passed_pawns_score, 45);
    }

    #[test]
    fn test_isolated_pawns() {
        let pos = Position::new();
        let white_material = Evaluation::get_material_info(&pos, Color::WHITE);
        let black_material = Evaluation::get_material_info(&pos, Color::BLACK);
        assert_eq!(Evaluation::evaluate_isolated_pawns(&white_material), 0);
        assert_eq!(Evaluation::evaluate_isolated_pawns(&black_material), 0);

        let pos = Position::from_fen("rnbqkbnr/pp3ppp/8/1P1p2P1/6P1/8/3PP3/RNBQKBNR b KQkq - 0 1")
            .unwrap();
        let white_material = Evaluation::get_material_info(&pos, Color::WHITE);
        let black_material = Evaluation::get_material_info(&pos, Color::BLACK);
        assert_eq!(Evaluation::evaluate_isolated_pawns(&white_material), -50);
        assert_eq!(Evaluation::evaluate_isolated_pawns(&black_material), -10);
    }
}
