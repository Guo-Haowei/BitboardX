use super::board::*;
use super::moves::*;
use super::piece::{Color, Piece, PieceType};
use super::position::Position;

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
fn pseudo_legal_impl<const ATTACK_ONLY: bool>(
    pos: &Position,
    sq: Square,
    color: Color,
) -> BitBoard {
    let piece = pos.get_piece(sq);

    match piece {
        Piece::W_PAWN => move_pawn::<{ Color::WHITE.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::B_PAWN => move_pawn::<{ Color::BLACK.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::W_ROOK | Piece::B_ROOK => move_sliding::<0, 4>(pos, sq, color),
        Piece::W_BISHOP | Piece::B_BISHOP => move_sliding::<4, 8>(pos, sq, color),
        Piece::W_QUEEN | Piece::B_QUEEN => move_sliding::<0, 8>(pos, sq, color),
        Piece::W_KNIGHT | Piece::B_KNIGHT => move_knight(pos, sq, color),
        Piece::W_KING => move_king::<{ Color::WHITE.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::B_KING => move_king::<{ Color::BLACK.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::NONE => BitBoard::new(),
        _ => {
            panic!("Invalid piece type: {:?}", piece);
        }
    }
}

pub fn pseudo_legal_move(pos: &Position, sq: Square) -> BitBoard {
    pseudo_legal_impl::<false>(pos, sq, pos.side_to_move)
}

pub fn pseudo_legal_attack(pos: &Position, sq: Square, color: Color) -> BitBoard {
    pseudo_legal_impl::<true>(pos, sq, color)
}

/// Pseudo-legal move generation for pawns
pub fn move_pawn<const COLOR: u8, const ATTACK_ONLY: bool>(pos: &Position, sq: Square) -> BitBoard {
    let (_file, rank) = sq.file_rank();
    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();

    let opponent = COLOR ^ 1;

    let is_white = COLOR == Color::WHITE.as_u8();
    let is_black = COLOR != Color::WHITE.as_u8();

    // Handle forward moves
    if !ATTACK_ONLY {
        let next_bb = if is_white { shift_north(bb) } else { shift_south(bb) };

        if (next_bb & pos.occupancies[Color::BOTH.as_usize()]).none() {
            moves |= next_bb;
        }

        if (is_white && rank == RANK_2 || is_black && rank == RANK_7) && moves.any() {
            let next_bb = if is_white { shift_north(next_bb) } else { shift_south(next_bb) };
            if (next_bb & pos.occupancies[Color::BOTH.as_usize()]).none() {
                moves |= next_bb;
            }
        }
    }

    // Handle attacks moves
    let attack_left = if is_white { shift_nw(bb) } else { shift_sw(bb) };
    if ATTACK_ONLY || (attack_left & pos.occupancies[opponent as usize]).any() {
        moves |= attack_left;
    }
    let attack_right = if is_white { shift_ne(bb) } else { shift_se(bb) };
    if ATTACK_ONLY || (attack_right & pos.occupancies[opponent as usize]).any() {
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
pub fn move_sliding<const START: u8, const END: u8>(
    pos: &Position,
    sq: Square,
    color: Color,
) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();
    let opponent = color.opponent();

    for i in START..END {
        let mut next_bb = SHIFT_FUNCS[i as usize](bb);

        while next_bb.any() {
            if (next_bb & pos.occupancies[color.as_usize()]).any() {
                break;
            }

            if (next_bb & pos.occupancies[opponent.as_usize()]).any() {
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
pub fn move_knight(pos: &Position, sq: Square, color: Color) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();
    let occupancy = !pos.occupancies[color.as_usize()];

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

fn min_max(a: Square, b: Square) -> (Square, Square) {
    if a.0 < b.0 { (a, b) } else { (b, a) }
}

/// Pseudo-legal move generation for a king
fn castling_check<const COLOR: u8>(
    pos: &Position,
    sq: Square,
    dst_sq: Square,
    rook_sq: Square,
) -> bool {
    // r . . . k . . r
    // a b c d e f g h
    let color = Color::from(COLOR);
    let opponent = color.opponent();

    // check if the rook is in the right place
    let rook_type = Piece::get_piece(color, PieceType::Rook);
    if pos.bitboards[rook_type.as_usize()].test(rook_sq.as_u8()) == false {
        return false;
    }

    // check if the cells are under attack
    let (start, end) = min_max(sq, dst_sq);
    for i in start.0..=end.0 {
        if pos.attack_map[opponent.as_usize()].test(i) {
            return false;
        }
    }

    // check if any piece is in the way
    let (start, end) = min_max(sq, rook_sq);
    for i in start.0 + 1..end.0 {
        if (pos.occupancies[Color::BOTH.as_usize()]).test(i) {
            return false;
        }
    }

    true
}

pub fn move_king<const COLOR: u8, const ATTACK_ONLY: bool>(pos: &Position, sq: Square) -> BitBoard {
    let color = Color::from(COLOR);
    let is_white = color.is_white();

    let bb = sq.to_bitboard();
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
        moves &= !pos.attack_map[color.opponent().as_usize()];

        // Castling doesn't form attack
        if is_white {
            if (pos.castling & MoveFlags::K != 0)
                && castling_check::<COLOR>(pos, sq, Square::G1, Square::H1)
            {
                assert!(sq == Square::E1);
                moves |= BB_G1;
            }
            if (pos.castling & MoveFlags::Q != 0)
                && castling_check::<COLOR>(pos, sq, Square::C1, Square::A1)
            {
                assert!(sq == Square::E1);
                moves |= BB_C1;
            }
        } else {
            if (pos.castling & MoveFlags::k != 0)
                && castling_check::<COLOR>(pos, sq, Square::G8, Square::H8)
            {
                assert!(sq == Square::E8);
                moves |= BB_G8;
            }
            if (pos.castling & MoveFlags::q != 0)
                && castling_check::<COLOR>(pos, sq, Square::C8, Square::A8)
            {
                assert!(sq == Square::E8);
                moves |= BB_C8;
            }
        }
    }

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

        let moves = pseudo_legal_move(&pos, Square::E2);
        assert_eq!(moves, BB_E3 | BB_E4);
    }

    #[test]
    fn test_black_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::D7);
        assert_eq!(moves, BB_D6 | BB_D5);
    }

    #[test]
    fn white_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::E4);
        assert_eq!(moves, BB_D5 | BB_E5);
    }

    #[test]
    fn black_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::D5);
        assert_eq!(moves, BB_D4 | BB_E4);
    }

    #[test]
    fn move_biship() {
        let fen = "rn1qkbnr/ppp1pppp/4b3/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::E6);
        assert_eq!(moves, BB_C8 | BB_D7 | BB_F5 | BB_G4 | BB_H3);
    }

    #[test]
    fn move_rook() {
        let fen = "rnbqkbn1/ppp1pp1r/7p/3p2p1/7P/3P1PPR/PPP1P3/RNBQKBN1 b - - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::H7);
        assert_eq!(moves, BB_H8 | BB_G7);
    }

    #[test]
    fn move_knight() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::B1);
        assert_eq!(moves, BB_A3 | BB_C3);
    }

    #[test]
    fn knight_attack() {
        let fen = "rn2kb1r/pppqppp1/5n2/3p3p/4P1b1/BPN5/P1PP1PPP/R2QKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::F6);
        assert_eq!(moves, BB_E4 | BB_G8 | BB_H7);
    }

    #[test]
    fn test_castling() {
        let fen = "r3k2r/ppp1bppp/2n1pn2/3p4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b kq - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::E8);
        let expected_moves = BB_C8 | BB_D8 | BB_F8 | BB_G8 | BB_D7;
        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_castling_2() {
        let fen = "8/4k3/8/8/8/8/r6R/R3K3 w Q - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move(&pos, Square::E1);
        assert!(moves.test(Square::C1.as_u8()))
    }
}

// @TODO: get rid of these constants
pub const BB_A1: BitBoard = Square::A1.to_bitboard();
pub const BB_A2: BitBoard = Square::A2.to_bitboard();
pub const BB_A3: BitBoard = Square::A3.to_bitboard();
pub const BB_A4: BitBoard = Square::A4.to_bitboard();
pub const BB_A5: BitBoard = Square::A5.to_bitboard();
pub const BB_A6: BitBoard = Square::A6.to_bitboard();
pub const BB_A7: BitBoard = Square::A7.to_bitboard();
pub const BB_A8: BitBoard = Square::A8.to_bitboard();

pub const BB_B1: BitBoard = Square::B1.to_bitboard();
pub const BB_B2: BitBoard = Square::B2.to_bitboard();
pub const BB_B3: BitBoard = Square::B3.to_bitboard();
pub const BB_B4: BitBoard = Square::B4.to_bitboard();
pub const BB_B5: BitBoard = Square::B5.to_bitboard();
pub const BB_B6: BitBoard = Square::B6.to_bitboard();
pub const BB_B7: BitBoard = Square::B7.to_bitboard();
pub const BB_B8: BitBoard = Square::B8.to_bitboard();

pub const BB_C1: BitBoard = Square::C1.to_bitboard();
pub const BB_C2: BitBoard = Square::C2.to_bitboard();
pub const BB_C3: BitBoard = Square::C3.to_bitboard();
pub const BB_C4: BitBoard = Square::C4.to_bitboard();
pub const BB_C5: BitBoard = Square::C5.to_bitboard();
pub const BB_C6: BitBoard = Square::C6.to_bitboard();
pub const BB_C7: BitBoard = Square::C7.to_bitboard();
pub const BB_C8: BitBoard = Square::C8.to_bitboard();

pub const BB_D1: BitBoard = Square::D1.to_bitboard();
pub const BB_D2: BitBoard = Square::D2.to_bitboard();
pub const BB_D3: BitBoard = Square::D3.to_bitboard();
pub const BB_D4: BitBoard = Square::D4.to_bitboard();
pub const BB_D5: BitBoard = Square::D5.to_bitboard();
pub const BB_D6: BitBoard = Square::D6.to_bitboard();
pub const BB_D7: BitBoard = Square::D7.to_bitboard();
pub const BB_D8: BitBoard = Square::D8.to_bitboard();

pub const BB_E1: BitBoard = Square::E1.to_bitboard();
pub const BB_E2: BitBoard = Square::E2.to_bitboard();
pub const BB_E3: BitBoard = Square::E3.to_bitboard();
pub const BB_E4: BitBoard = Square::E4.to_bitboard();
pub const BB_E5: BitBoard = Square::E5.to_bitboard();
pub const BB_E6: BitBoard = Square::E6.to_bitboard();
pub const BB_E7: BitBoard = Square::E7.to_bitboard();
pub const BB_E8: BitBoard = Square::E8.to_bitboard();

pub const BB_F1: BitBoard = Square::F1.to_bitboard();
pub const BB_F2: BitBoard = Square::F2.to_bitboard();
pub const BB_F3: BitBoard = Square::F3.to_bitboard();
pub const BB_F4: BitBoard = Square::F4.to_bitboard();
pub const BB_F5: BitBoard = Square::F5.to_bitboard();
pub const BB_F6: BitBoard = Square::F6.to_bitboard();
pub const BB_F7: BitBoard = Square::F7.to_bitboard();
pub const BB_F8: BitBoard = Square::F8.to_bitboard();

pub const BB_G1: BitBoard = Square::G1.to_bitboard();
pub const BB_G2: BitBoard = Square::G2.to_bitboard();
pub const BB_G3: BitBoard = Square::G3.to_bitboard();
pub const BB_G4: BitBoard = Square::G4.to_bitboard();
pub const BB_G5: BitBoard = Square::G5.to_bitboard();
pub const BB_G6: BitBoard = Square::G6.to_bitboard();
pub const BB_G7: BitBoard = Square::G7.to_bitboard();
pub const BB_G8: BitBoard = Square::G8.to_bitboard();

pub const BB_H1: BitBoard = Square::H1.to_bitboard();
pub const BB_H2: BitBoard = Square::H2.to_bitboard();
pub const BB_H3: BitBoard = Square::H3.to_bitboard();
pub const BB_H4: BitBoard = Square::H4.to_bitboard();
pub const BB_H5: BitBoard = Square::H5.to_bitboard();
pub const BB_H6: BitBoard = Square::H6.to_bitboard();
pub const BB_H7: BitBoard = Square::H7.to_bitboard();
pub const BB_H8: BitBoard = Square::H8.to_bitboard();
