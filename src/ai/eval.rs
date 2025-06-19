use crate::core::{position::Position, types::*};

const GAME_PHASE_INC: [i32; 12] = [0, 0, 1, 1, 1, 1, 2, 2, 4, 4, 0, 0];

const GAME_PHASE_FACTOR: i32 = 24; // max game phase factor

fn evaluate_pesto(pos: &Position) -> i32 {
    use super::pesto::{EG_TABLE, MG_TABLE};

    let mut phase = 0;
    let mut mg = [0; 2]; // middle game score
    let mut eg = [0; 2]; // end game score

    for piece_idx in 0..Piece::COUNT {
        let bitboard = pos.bitboards[piece_idx];
        let color_idx = piece_idx / PieceType::COUNT as usize;
        for sq in bitboard.iter() {
            let sq_idx = sq.as_u8() as usize;
            mg[color_idx] += MG_TABLE[piece_idx][sq_idx];
            eg[color_idx] += EG_TABLE[piece_idx][sq_idx];
            phase += GAME_PHASE_INC[piece_idx];
        }
    }

    let mg_score = mg[0] - mg[1];
    let eg_score = eg[0] - eg[1];
    let mg_phase = phase.min(GAME_PHASE_FACTOR); // clamp to 24
    let eg_phase = GAME_PHASE_FACTOR - mg_phase;

    /* tapered eval */
    let score = (mg_score * mg_phase + eg_score * eg_phase) / GAME_PHASE_FACTOR;
    score
}

// @TODO: Implement pawn structure, mobility, rook activity, bishop pair, etc.
fn evaluate_king_safety(_pos: &Position) -> i32 {
    // Placeholder for king safety evaluation
    0
}

// for simplicity, we use mid-game piece value table
fn get_piece_value(piece_type: PieceType) -> i32 {
    assert!(piece_type != PieceType::NONE, "Piece must not be NONE");
    use super::pesto::MG_PIECE_VALUE;

    MG_PIECE_VALUE[piece_type.as_u8() as usize]
}

pub fn evaluate(pos: &Position) -> i32 {
    debug_assert!(pos.side_to_move == Color::WHITE || pos.side_to_move == Color::BLACK);
    let mut score = 0;

    score += evaluate_pesto(pos);
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
