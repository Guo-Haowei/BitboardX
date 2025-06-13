use super::bitboard::BitBoard;
use super::moves::*;
use super::piece::*;
use super::position::Position;
use super::types::*;

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

/// Pseudo-legal move generation for a square
fn pseudo_legal_impl<const ATTACK_ONLY: bool>(pos: &Position, sq: u8, color: u8) -> BitBoard {
    let piece = pos.get_piece(sq);

    match piece {
        Piece::W_PAWN => move_pawn::<COLOR_WHITE, ATTACK_ONLY>(pos, sq),
        Piece::B_PAWN => move_pawn::<COLOR_BLACK, ATTACK_ONLY>(pos, sq),
        Piece::W_ROOK | Piece::B_ROOK => move_sliding::<0, 4>(pos, sq, color),
        Piece::W_BISHOP | Piece::B_BISHOP => move_sliding::<4, 8>(pos, sq, color),
        Piece::W_QUEEN | Piece::B_QUEEN => move_sliding::<0, 8>(pos, sq, color),
        Piece::W_KNIGHT | Piece::B_KNIGHT => move_knight(pos, sq, color),
        Piece::W_KING => move_king::<COLOR_WHITE, ATTACK_ONLY>(pos, sq),
        Piece::B_KING => move_king::<COLOR_BLACK, ATTACK_ONLY>(pos, sq),
        Piece::NONE => BitBoard::new(),
        _ => {
            panic!("Invalid piece type: {:?}", piece);
        }
    }
}

pub fn pseudo_legal_move(pos: &Position, sq: u8) -> BitBoard {
    pseudo_legal_impl::<false>(pos, sq, pos.side_to_move)
}

pub fn pseudo_legal_attack(pos: &Position, sq: u8, color: u8) -> BitBoard {
    pseudo_legal_impl::<true>(pos, sq, color)
}

/// Pseudo-legal move generation for pawns
pub fn move_pawn<const COLOR: u8, const ATTACK_ONLY: bool>(pos: &Position, sq: u8) -> BitBoard {
    let (_file, rank) = get_file_rank(sq);
    let bb = BitBoard::from_bit(sq);
    let mut moves = BitBoard::new();

    let opposite_color = get_opposite_color(COLOR);

    let is_white = COLOR == COLOR_WHITE;
    let is_black = COLOR != COLOR_WHITE;

    // Handle forward moves
    if !ATTACK_ONLY {
        let next_bb = if is_white { shift_north(bb) } else { shift_south(bb) };

        if (next_bb & pos.occupancies[COLOR_BOTH as usize]).none() {
            moves |= next_bb;
        }

        if (is_white && rank == RANK_2 || is_black && rank == RANK_7) && moves.any() {
            let next_bb = if is_white { shift_north(next_bb) } else { shift_south(next_bb) };
            if (next_bb & pos.occupancies[COLOR_BOTH as usize]).none() {
                moves |= next_bb;
            }
        }
    }

    // Handle attacks moves
    let attack_left = if is_white { shift_nw(bb) } else { shift_sw(bb) };
    if ATTACK_ONLY || (attack_left & pos.occupancies[opposite_color as usize]).any() {
        moves |= attack_left;
    }
    let attack_right = if is_white { shift_ne(bb) } else { shift_se(bb) };
    if ATTACK_ONLY || (attack_right & pos.occupancies[opposite_color as usize]).any() {
        moves |= attack_right;
    }

    // @TODO: handle en passant - BEGIN
    // @TODO: handle en passant - END

    // @TODO: handle promotion - BEGIN
    {
        let mut promotion = false;
        match rank {
            RANK_7 if is_white => {
                promotion = true;
            }
            RANK_2 if is_black => {
                promotion = true;
            }
            _ => {}
        }
        if promotion {
            println!("TODO: handle promotion for pawn at square {}", sq);
        }
    }
    // @TODO: handle promotion - END

    moves
}

/// Pseudo-legal move generation for a sliding piece (rook, bishop, queen)
pub fn move_sliding<const START: u8, const END: u8>(pos: &Position, sq: u8, color: u8) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = BitBoard::from_bit(sq);
    let opposite_color = get_opposite_color(color);

    for i in START..END {
        let mut next_bb = SHIFT_FUNCS[i as usize](bb);

        while next_bb.any() {
            if (next_bb & pos.occupancies[color as usize]).any() {
                break;
            }

            if (next_bb & pos.occupancies[opposite_color as usize]).any() {
                moves |= next_bb;
                break;
            }

            moves |= next_bb;

            next_bb = SHIFT_FUNCS[i as usize](next_bb);
        }
    }

    moves
}

/// Pseudo-legal move generation for a knight
pub fn move_knight(pos: &Position, sq: u8, color: u8) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = BitBoard::from_bit(sq);
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

/// Pseudo-legal move generation for a king
fn castling_check<const COLOR: u8>(pos: &Position, sq: u8, dst_sq: u8, rook_sq: u8) -> bool {
    // r . . . k . . r
    // a b c d e f g h
    let opponent = get_opposite_color(COLOR);

    // check if the rook is in the right place
    let rook_type = Piece::get_piece(COLOR, PieceType::Rook);
    if pos.bitboards[rook_type.as_usize()].test(rook_sq) == false {
        return false;
    }

    // check if the cells are under attack
    let start = sq.min(dst_sq);
    let end = sq.max(dst_sq);
    for i in start..=end {
        if pos.attack_map[opponent as usize].test(i) {
            return false;
        }
    }

    // check if any piece is in the way
    let start = sq.min(rook_sq);
    let end = sq.max(rook_sq);
    for i in start + 1..end {
        if (pos.occupancies[COLOR_BOTH as usize]).test(i) {
            return false;
        }
    }

    true
}

pub fn move_king<const COLOR: u8, const ATTACK_ONLY: bool>(pos: &Position, sq: u8) -> BitBoard {
    let is_white = COLOR == COLOR_WHITE;

    let bb = BitBoard::from_bit(sq);
    let mut moves = BitBoard::new();
    let occupancy = !pos.occupancies[COLOR as usize];
    moves |= shift_north(bb) & occupancy;
    moves |= shift_south(bb) & occupancy;
    moves |= shift_east(bb) & occupancy;
    moves |= shift_west(bb) & occupancy;
    moves |= shift_ne(bb) & occupancy;
    moves |= shift_nw(bb) & occupancy;
    moves |= shift_se(bb) & occupancy;
    moves |= shift_sw(bb) & occupancy;
    if !ATTACK_ONLY {
        // If we are checking if cells are being attacked, not actually moving, no need exclude pieces under attack
        moves &= !pos.attack_map[get_opposite_color(COLOR) as usize];

        // Castling doesn't form attack
        if is_white {
            if (pos.castling & MoveFlags::K != 0) && castling_check::<COLOR>(pos, sq, SQ_G1, SQ_H1)
            {
                assert!(sq == SQ_E1);
                moves |= BB_G1;
            }
            if (pos.castling & MoveFlags::Q != 0) && castling_check::<COLOR>(pos, sq, SQ_C1, SQ_A1)
            {
                assert!(sq == SQ_E1);
                moves |= BB_C1;
            }
        } else {
            if (pos.castling & MoveFlags::k != 0) && castling_check::<COLOR>(pos, sq, SQ_G8, SQ_H8)
            {
                assert!(sq == SQ_E8);
                moves |= BB_G8;
            }
            if (pos.castling & MoveFlags::q != 0) && castling_check::<COLOR>(pos, sq, SQ_C8, SQ_A8)
            {
                assert!(sq == SQ_E8);
                moves |= BB_C8;
            }
        }
    }

    println!("bitboard:\n{}", moves);
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );

        let moves = pseudo_legal_move(&pos, SQ_E2);
        assert_eq!(moves, BB_E3 | BB_E4);
    }

    #[test]
    fn test_black_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_D7);
        assert_eq!(moves, BB_D6 | BB_D5);
    }

    #[test]
    fn white_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_E4);
        assert_eq!(moves, BB_D5 | BB_E5);
    }

    #[test]
    fn black_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_D5);
        assert_eq!(moves, BB_D4 | BB_E4);
    }

    #[test]
    fn move_biship() {
        let fen = "rn1qkbnr/ppp1pppp/4b3/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_E6);
        assert_eq!(moves, BB_C8 | BB_D7 | BB_F5 | BB_G4 | BB_H3);
    }

    #[test]
    fn move_rook() {
        let fen = "rnbqkbn1/ppp1pp1r/7p/3p2p1/7P/3P1PPR/PPP1P3/RNBQKBN1 b - - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_H7);
        assert_eq!(moves, BB_H8 | BB_G7);
    }

    #[test]
    fn move_knight() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_B1);
        assert_eq!(moves, BB_A3 | BB_C3);
    }

    #[test]
    fn knight_attack() {
        let fen = "rn2kb1r/pppqppp1/5n2/3p3p/4P1b1/BPN5/P1PP1PPP/R2QKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_F6);
        assert_eq!(moves, BB_E4 | BB_G8 | BB_H7);
    }

    #[test]
    fn test_castling() {
        let fen = "r3k2r/ppp1bppp/2n1pn2/3p4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b kq - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_E8);
        let expected_moves = BB_C8 | BB_D8 | BB_F8 | BB_G8 | BB_D7;
        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_castling_2() {
        let fen = "8/4k3/8/8/8/8/r6R/R3K3 w Q - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, SQ_E1);
        assert!(moves.test(SQ_C1))
    }
}
