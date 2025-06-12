use crate::board::bitboard::BitBoard;
use crate::board::moves::MoveFlags;
use crate::board::position::*;
use crate::board::types::*;

const NORTH: i32 = 8;
const SOUTH: i32 = -NORTH;
const EAST: i32 = 1;
const WEST: i32 = -EAST;
const NE: i32 = NORTH + EAST;
const NW: i32 = NORTH + WEST;
const SE: i32 = SOUTH + EAST;
const SW: i32 = SOUTH + WEST;

const BOUND_A: BitBoard = BitBoard::from(0x0101010101010101);
const BOUND_B: BitBoard = BitBoard::from(0x0202020202020202);
const BOUND_G: BitBoard = BitBoard::from(0x4040404040404040);
const BOUND_H: BitBoard = BitBoard::from(0x8080808080808080);
const BOUND_1: BitBoard = BitBoard::from(0x00000000000000FF);
const BOUND_2: BitBoard = BitBoard::from(0x000000000000FF00);
const BOUND_7: BitBoard = BitBoard::from(0x00FF000000000000);
const BOUND_8: BitBoard = BitBoard::from(0xFF00000000000000);
const BOUND_AB: BitBoard = BitBoard::from(BOUND_A.get() | BOUND_B.get());
const BOUND_GH: BitBoard = BitBoard::from(BOUND_G.get() | BOUND_H.get());
const BOUND_12: BitBoard = BitBoard::from(BOUND_1.get() | BOUND_2.get());
const BOUND_78: BitBoard = BitBoard::from(BOUND_7.get() | BOUND_8.get());

fn shift(bb: BitBoard, dir: i32) -> BitBoard {
    // if dir > 0 { bb.get() << dir } else { bb.get() >> -dir }
    BitBoard::from(if dir < 0 { bb.get() >> -dir } else { bb.get() << dir })
}

fn shift_east(bb: BitBoard) -> BitBoard {
    shift(bb & !BOUND_H, EAST)
}

fn shift_west(bb: BitBoard) -> BitBoard {
    shift(bb & !BOUND_A, WEST)
}

fn shift_north(bb: BitBoard) -> BitBoard {
    shift(bb & !BOUND_8, NORTH)
}

fn shift_south(bb: BitBoard) -> BitBoard {
    shift(bb & !BOUND_1, SOUTH)
}

fn shift_ne(bb: BitBoard) -> BitBoard {
    shift(bb & !(BOUND_H | BOUND_8), NE)
}

fn shift_nw(bb: BitBoard) -> BitBoard {
    shift(bb & !(BOUND_A | BOUND_8), NW)
}

fn shift_se(bb: BitBoard) -> BitBoard {
    shift(bb & !(BOUND_H | BOUND_1), SE)
}

fn shift_sw(bb: BitBoard) -> BitBoard {
    shift(bb & !(BOUND_A | BOUND_1), SW)
}

const SHIFT_FUNCS: [fn(BitBoard) -> BitBoard; 8] =
    [shift_north, shift_south, shift_east, shift_west, shift_ne, shift_nw, shift_se, shift_sw];

fn move_sliding<const START: u8, const END: u8>(pos: &Position, file: u8, rank: u8, color: Color) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = BitBoard::from_bit(make_square(file, rank));
    let opposite_color = get_opposite_color(color);

    for i in START..END {
        let mut new_pos = SHIFT_FUNCS[i as usize](bb);

        while new_pos.has_any() {
            if (new_pos & pos.occupancies[color as usize]).has_any() {
                break;
            }

            if (new_pos & pos.occupancies[opposite_color as usize]).has_any() {
                moves |= new_pos;
                break;
            }

            moves |= new_pos;
            new_pos = SHIFT_FUNCS[i as usize](new_pos);
        }
    }

    moves
}

fn castling_helper<const IS_WHITE: bool, const ROOK_SQ: u8, const BLOCK: u64, const ATTACK: u64>(
    pos: &Position,
) -> bool {
    let opposite_color = if !IS_WHITE { Color::White } else { Color::Black };
    let occupancy = pos.occupancies[Color::Both as usize];

    // check if rook is taken out
    if IS_WHITE && !pos.state.bitboards[Piece::WRook as usize].has_bit(ROOK_SQ) {
        return false;
    } else if !IS_WHITE && !pos.state.bitboards[Piece::BRook as usize].has_bit(ROOK_SQ) {
        return false;
    }

    // @TODO: bitset
    let can_castle = match ROOK_SQ {
        SQ_H1 => pos.state.castling & MoveFlags::K != 0,
        SQ_H8 => pos.state.castling & MoveFlags::k != 0,
        SQ_A1 => pos.state.castling & MoveFlags::Q != 0,
        SQ_A8 => pos.state.castling & MoveFlags::q != 0,
        _ => panic!("Invalid castling square: {}", ROOK_SQ),
    };

    if !can_castle {
        return false;
    }

    if (BitBoard::from(BLOCK) & occupancy).has_any() {
        return false;
    }

    if (BitBoard::from(ATTACK) & pos.attack_map[opposite_color as usize]).has_any() {
        return false;
    }

    true
}

fn move_king<const IS_WHITE: bool, const ATTACK_ONLY: bool>(pos: &Position, file: u8, rank: u8) -> BitBoard {
    let color = if IS_WHITE { Color::White } else { Color::Black };
    let opposite_color = if !IS_WHITE { Color::White } else { Color::Black };
    let bb = BitBoard::from_bit(make_square(file, rank));
    let mut moves = BitBoard::new();
    let occupancy = !pos.occupancies[color as usize];
    moves |= shift_north(bb) & occupancy;
    moves |= shift_south(bb) & occupancy;
    moves |= shift_east(bb) & occupancy;
    moves |= shift_west(bb) & occupancy;
    moves |= shift_ne(bb) & occupancy;
    moves |= shift_nw(bb) & occupancy;
    moves |= shift_se(bb) & occupancy;
    moves |= shift_sw(bb) & occupancy;
    // If we are checking if cells are being attacked, not actually moving, no need exclude pieces under attack
    if !ATTACK_ONLY {
        moves &= !pos.attack_map[opposite_color as usize];
    }

    // Castling
    if IS_WHITE {
        const BLOCK_KS: u64 = (1u64 << SQ_F1) | (1u64 << SQ_G1);
        const ATTACK_KS: u64 = (1u64 << SQ_E1) | (1u64 << SQ_F1) | (1u64 << SQ_G1);
        let king_side = castling_helper::<true, SQ_H1, BLOCK_KS, ATTACK_KS>(pos);
        if king_side {
            moves |= BB_G1;
        }
        const BLOCK_QS: u64 = (1u64 << SQ_B1) | (1u64 << SQ_C1) | (1u64 << SQ_D1);
        const ATTACK_QS: u64 = (1u64 << SQ_C1) | (1u64 << SQ_D1) | (1u64 << SQ_E1);
        let queen_side = castling_helper::<true, SQ_A1, BLOCK_QS, ATTACK_QS>(pos);
        if queen_side {
            moves |= BB_C1;
        }
    } else {
        const BLOCK_KS: u64 = (1u64 << SQ_F8) | (1u64 << SQ_G8);
        const ATTACK_KS: u64 = (1u64 << SQ_E8) | BLOCK_KS;
        let king_side = castling_helper::<false, SQ_H8, BLOCK_KS, ATTACK_KS>(pos);
        if king_side {
            moves |= BB_G8;
        }
        const BLOCK_QS: u64 = (1u64 << SQ_B8) | (1u64 << SQ_C8) | (1u64 << SQ_D8);
        const ATTACK_QS: u64 = (1u64 << SQ_E8) | BLOCK_QS;
        let queen_side = castling_helper::<false, SQ_A8, BLOCK_QS, ATTACK_QS>(pos);
        if queen_side {
            moves |= BB_C8;
        }
    }

    moves
}

fn move_knight(pos: &Position, file: u8, rank: u8, color: Color) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = BitBoard::from_bit(make_square(file, rank));
    let occupancy = !pos.occupancies[color as usize];

    moves |= shift(bb & !(BOUND_AB | BOUND_1), SW + WEST) & occupancy;
    moves |= shift(bb & !(BOUND_AB | BOUND_8), NW + WEST) & occupancy;
    moves |= shift(bb & !(BOUND_GH | BOUND_1), SE + EAST) & occupancy;
    moves |= shift(bb & !(BOUND_GH | BOUND_8), NE + EAST) & occupancy;

    moves |= shift(bb & !(BOUND_A | BOUND_12), SW + SOUTH) & occupancy;
    moves |= shift(bb & !(BOUND_A | BOUND_78), NW + NORTH) & occupancy;
    moves |= shift(bb & !(BOUND_H | BOUND_12), SE + SOUTH) & occupancy;
    moves |= shift(bb & !(BOUND_H | BOUND_78), NE + NORTH) & occupancy;

    moves
}

fn move_pawn<const IS_WHITE: bool, const ATTACK_ONLY: bool>(pos: &Position, file: u8, rank: u8) -> BitBoard {
    let bb = BitBoard::from_bit(make_square(file, rank));
    let mut moves = BitBoard::new();
    let opposite_color = if IS_WHITE { Color::Black } else { Color::White };

    // Promotion
    if (IS_WHITE && rank == RANK_7) || (!IS_WHITE && rank == RANK_2) {
        eprintln!("TODO: Handle promotion");
    }

    // Move forward
    if !ATTACK_ONLY {
        let new_pos_1 = if IS_WHITE { shift_north(bb) } else { shift_south(bb) };

        if (new_pos_1 & pos.occupancies[Color::Both as usize]).is_empty() {
            moves |= new_pos_1;
        }

        if (IS_WHITE && rank == RANK_2 || !IS_WHITE && rank == RANK_7) && moves.has_any() {
            let new_pos_2 = if IS_WHITE { shift_north(new_pos_1) } else { shift_south(new_pos_1) };
            if (new_pos_2 & pos.occupancies[Color::Both as usize]).is_empty() {
                moves |= new_pos_2;
            }
        }
    }

    // Attack
    let attack_left = if IS_WHITE { shift_nw(bb) } else { shift_sw(bb) };
    if ATTACK_ONLY || (attack_left & pos.occupancies[opposite_color as usize]).has_any() {
        moves |= attack_left;
    }
    let attack_right = if IS_WHITE { shift_ne(bb) } else { shift_se(bb) };
    if ATTACK_ONLY || (attack_right & pos.occupancies[opposite_color as usize]).has_any() {
        moves |= attack_right;
    }

    moves
}

pub fn gen_moves(pos: &Position, square: u8) -> BitBoard {
    gen_moves_impl::<false>(pos, square, pos.state.side_to_move)
}

pub fn gen_attack_moves(pos: &Position, square: u8, color: Color) -> BitBoard {
    gen_moves_impl::<true>(pos, square, color)
}

fn gen_moves_impl<const ATTACK_ONLY: bool>(pos: &Position, square: u8, color: Color) -> BitBoard {
    let (file, rank) = get_file_rank(square);

    let start = if color == Color::White { W_START } else { B_START };
    let end = if color == Color::White { W_END } else { B_END };
    for i in start..end {
        if !pos.state.bitboards[i as usize].has_bit(square) {
            continue;
        }

        let piece: Piece = unsafe { std::mem::transmute(i as u8) };

        return match piece {
            Piece::WPawn => move_pawn::<true, ATTACK_ONLY>(pos, file, rank),
            Piece::BPawn => move_pawn::<false, ATTACK_ONLY>(pos, file, rank),
            Piece::WRook | Piece::BRook => move_sliding::<0, 4>(pos, file, rank, color),
            Piece::WBishop | Piece::BBishop => move_sliding::<4, 8>(pos, file, rank, color),
            Piece::WQueen | Piece::BQueen => move_sliding::<0, 8>(pos, file, rank, color),
            Piece::WKnight | Piece::BKnight => move_knight(pos, file, rank, color),
            Piece::WKing => move_king::<true, ATTACK_ONLY>(pos, file, rank),
            Piece::BKing => move_king::<false, ATTACK_ONLY>(pos, file, rank),
            _ => panic!("Unsupported piece type for move generation: {:?}", piece),
        };
    }

    BitBoard::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_white_pawn() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        assert_eq!(pos.state.to_board_string(), "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR");

        let moves = gen_moves(&pos, SQ_E2);
        assert_eq!(moves, BB_E3 | BB_E4);
    }

    #[test]
    fn move_black_pawn() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_D7);
        assert_eq!(moves, BB_D6 | BB_D5);
    }

    #[test]
    fn white_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_E4);
        assert_eq!(moves, BB_D5 | BB_E5);
    }

    #[test]
    fn black_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_D5);
        assert_eq!(moves, BB_D4 | BB_E4);
    }

    #[test]
    fn move_biship() {
        let fen = "rn1qkbnr/ppp1pppp/4b3/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_E6);
        assert_eq!(moves, BB_C8 | BB_D7 | BB_F5 | BB_G4 | BB_H3);
    }

    #[test]
    fn move_rook() {
        let fen = "rnbqkbn1/ppp1pp1r/7p/3p2p1/7P/3P1PPR/PPP1P3/RNBQKBN1 b - - 0 1";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_H7);
        assert_eq!(moves, BB_H8 | BB_G7);
    }

    #[test]
    fn move_knight() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_B1);
        assert_eq!(moves, BB_A3 | BB_C3);
    }

    #[test]
    fn knight_attack() {
        let fen = "rn2kb1r/pppqppp1/5n2/3p3p/4P1b1/BPN5/P1PP1PPP/R2QKBNR b KQkq - 0 1";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_F6);
        assert_eq!(moves, BB_E4 | BB_G8 | BB_H7);
    }

    #[test]
    fn test_castling() {
        let fen = "r3k2r/ppp1bppp/2n1pn2/3p4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b kq - 0 10";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_E8);
        let expected_moves = BB_C8 | BB_D8 | BB_F8 | BB_G8 | BB_D7;
        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_castling_2() {
        let fen = "8/4k3/8/8/8/8/r6R/R3K3 w Q - 0 1";
        let pos = Position::from_fen(fen).unwrap();

        let moves = gen_moves(&pos, SQ_E1);
        assert!(moves.has_bit(SQ_C1))
    }
}
