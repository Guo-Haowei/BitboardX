use crate::board::Board;
use crate::types::*;

const NORTH: i32 = 8;
const SOUTH: i32 = -8;
const EAST: i32 = 1;
const WEST: i32 = -1;
const NE: i32 = NORTH + EAST;
const NW: i32 = NORTH + WEST;
const SE: i32 = SOUTH + EAST;
const SW: i32 = SOUTH + WEST;

const BOUND_A: u64 = 0x0101010101010101;
const BOUND_B: u64 = 0x0202020202020202;
const BOUND_G: u64 = 0x4040404040404040;
const BOUND_H: u64 = 0x8080808080808080;
const BOUND_1: u64 = 0x00000000000000FF;
const BOUND_2: u64 = 0x000000000000FF00;
const BOUND_7: u64 = 0x00FF000000000000;
const BOUND_8: u64 = 0xFF00000000000000;
const BOUND_AB: u64 = BOUND_A | BOUND_B;
const BOUND_GH: u64 = BOUND_G | BOUND_H;
const BOUND_12: u64 = BOUND_1 | BOUND_2;
const BOUND_78: u64 = BOUND_7 | BOUND_8;

fn shift(bb: u64, dir: i32) -> u64 {
    if dir > 0 { bb << dir } else { bb >> -dir }
}

fn shift_east(bb: u64) -> u64 {
    shift(bb & !BOUND_H, EAST)
}

fn shift_west(bb: u64) -> u64 {
    shift(bb & !BOUND_A, WEST)
}

fn shift_north(bb: u64) -> u64 {
    shift(bb & !BOUND_8, NORTH)
}

fn shift_south(bb: u64) -> u64 {
    shift(bb & !BOUND_1, SOUTH)
}

fn shift_ne(bb: u64) -> u64 {
    shift(bb & !(BOUND_H | BOUND_8), NE)
}

fn shift_nw(bb: u64) -> u64 {
    shift(bb & !(BOUND_A | BOUND_8), NW)
}

fn shift_se(bb: u64) -> u64 {
    shift(bb & !(BOUND_H | BOUND_1), SE)
}

fn shift_sw(bb: u64) -> u64 {
    shift(bb & !(BOUND_A | BOUND_1), SW)
}

const SHIFT_FUNCS: [fn(u64) -> u64; 8] =
    [shift_north, shift_south, shift_east, shift_west, shift_ne, shift_nw, shift_se, shift_sw];

pub fn parse_move(input: &str) -> Option<(u8, u8)> {
    if input.len() != 4 {
        return None;
    }

    let from_file = input.chars().nth(0)? as u8 - b'a';
    let from_rank = input.chars().nth(1)? as u8 - b'1';
    let to_file = input.chars().nth(2)? as u8 - b'a';
    let to_rank = input.chars().nth(3)? as u8 - b'1';

    if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
        return None;
    }

    Some((make_square(from_file, from_rank), make_square(to_file, to_rank)))
}

fn move_sliding<const START: u8, const END: u8>(board: &Board, file: u8, rank: u8, color: Color) -> u64 {
    let mut moves = 0u64;
    let bb = 1u64 << make_square(file, rank);
    let opposite_color = get_opposite_color(color);

    for i in START..END {
        let mut new_pos = SHIFT_FUNCS[i as usize](bb);

        while new_pos != 0 {
            if new_pos & board.occupancies[color as usize] != 0 {
                break;
            }

            if new_pos & board.occupancies[opposite_color as usize] != 0 {
                moves |= new_pos;
                break;
            }

            moves |= new_pos;
            new_pos = SHIFT_FUNCS[i as usize](new_pos);
        }
    }

    moves
}

fn move_king<const IS_WHITE: bool>(board: &Board, file: u8, rank: u8, color: Color) -> u64 {
    let bb = 1u64 << make_square(file, rank);
    let mut moves = 0u64;
    let occupancy = !board.occupancies[color as usize];
    moves |= shift_north(bb) & occupancy;
    moves |= shift_south(bb) & occupancy;
    moves |= shift_east(bb) & occupancy;
    moves |= shift_west(bb) & occupancy;
    moves |= shift_ne(bb) & occupancy;
    moves |= shift_nw(bb) & occupancy;
    moves |= shift_se(bb) & occupancy;
    moves |= shift_sw(bb) & occupancy;

    // Castling
    if IS_WHITE {
        if (board.castling & Castling::WK.bits()) != 0 {
            moves |= (1u64 << SQ_G1) & occupancy;
        }
        if (board.castling & Castling::WQ.bits()) != 0 {
            moves |= (1u64 << SQ_C1) & occupancy;
        }
    } else {
        if (board.castling & Castling::BK.bits()) != 0 {
            moves |= (1u64 << SQ_G8) & occupancy;
        }
        if (board.castling & Castling::BQ.bits()) != 0 {
            moves |= (1u64 << SQ_C8) & occupancy;
        }
    }

    moves
}

fn move_knight(board: &Board, file: u8, rank: u8, color: Color) -> u64 {
    let mut moves = 0u64;
    let bb = 1u64 << make_square(file, rank);
    let occupancy = !board.occupancies[color as usize];

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

fn move_pawn<const IS_WHITE: bool>(board: &Board, file: u8, rank: u8) -> u64 {
    let bb = 1u64 << make_square(file, rank);
    let mut moves = 0u64;
    let opposite_color = if IS_WHITE { Color::Black } else { Color::White };

    // Promotion
    if IS_WHITE && rank == RANK_7 || !IS_WHITE && rank == RANK_2 {
        println!("TODO: Handle promotion");
    }

    // Move forward
    let new_pos_1 = if IS_WHITE { shift_north(bb) } else { shift_south(bb) };

    if new_pos_1 & board.occupancies[2] == 0 {
        moves |= new_pos_1;
    }

    if (IS_WHITE && rank == RANK_2 || !IS_WHITE && rank == RANK_7) && moves != 0 {
        let new_pos_2 = if IS_WHITE { shift_north(new_pos_1) } else { shift_south(new_pos_1) };
        if new_pos_2 & board.occupancies[2] == 0 {
            moves |= new_pos_2;
        }
    }

    // Attack
    let attack_left = if IS_WHITE { shift_nw(bb) } else { shift_sw(bb) };
    if attack_left & board.occupancies[opposite_color as usize] != 0 {
        moves |= attack_left;
    }
    let attack_right = if IS_WHITE { shift_ne(bb) } else { shift_se(bb) };
    if attack_right & board.occupancies[opposite_color as usize] != 0 {
        moves |= attack_right;
    }

    moves
}

pub fn gen_moves(board: &Board, square: u8) -> u64 {
    let (file, rank) = get_file_rank(square);
    let mask = 1u64 << square;

    for i in 0..board.bitboards.len() {
        if (board.bitboards[i] & mask) == 0 {
            continue;
        }

        let piece: Piece = unsafe { std::mem::transmute(i as u8) };
        let color = if piece <= Piece::WhiteKing { Color::White } else { Color::Black };
        if color != board.side_to_move {
            return 0;
        }

        return match piece {
            Piece::WhitePawn => move_pawn::<true>(board, file, rank),
            Piece::BlackPawn => move_pawn::<false>(board, file, rank),
            Piece::WhiteRook | Piece::BlackRook => move_sliding::<0, 4>(board, file, rank, color),
            Piece::WhiteBishop | Piece::BlackBishop => move_sliding::<4, 8>(board, file, rank, color),
            Piece::WhiteQueen | Piece::BlackQueen => move_sliding::<0, 8>(board, file, rank, color),
            Piece::WhiteKnight | Piece::BlackKnight => move_knight(board, file, rank, color),
            Piece::WhiteKing => move_king::<true>(board, file, rank, color),
            Piece::BlackKing => move_king::<false>(board, file, rank, color),
            _ => 0,
        };
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_white_pawn() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_E2);
        assert_eq!(moves, (1u64 << SQ_E3) | (1u64 << SQ_E4));
    }

    #[test]
    fn move_black_pawn() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_D7);
        assert_eq!(moves, (1u64 << SQ_D6) | (1u64 << SQ_D5));
    }

    #[test]
    fn white_pawn_attack() {
        let mut board = Board::new();
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_E4);
        assert_eq!(moves, (1u64 << SQ_D5) | (1u64 << SQ_E5));
    }

    #[test]
    fn black_pawn_attack() {
        let mut board = Board::new();
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_D5);
        assert_eq!(moves, (1u64 << SQ_D4) | (1u64 << SQ_E4));
    }

    #[test]
    fn move_biship() {
        let mut board = Board::new();
        let fen = "rn1qkbnr/ppp1pppp/4b3/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_E6);
        assert_eq!(moves, (1u64 << SQ_C8) | (1u64 << SQ_D7) | (1u64 << SQ_F5) | (1u64 << SQ_G4) | (1u64 << SQ_H3));
    }

    #[test]
    fn move_rook() {
        let mut board = Board::new();
        let fen = "rnbqkbn1/ppp1pp1r/7p/3p2p1/7P/3P1PPR/PPP1P3/RNBQKBN1 b - - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_H7);
        assert_eq!(moves, (1u64 << SQ_H8) | (1u64 << SQ_G7));
    }

    #[test]
    fn move_knight() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_B1);
        assert_eq!(moves, (1u64 << SQ_A3) | (1u64 << SQ_C3));
    }

    #[test]
    fn knight_attack() {
        let mut board = Board::new();
        let fen = "rn2kb1r/pppqppp1/5n2/3p3p/4P1b1/BPN5/P1PP1PPP/R2QKBNR b KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_F6);
        assert_eq!(moves, (1u64 << SQ_E4) | (1u64 << SQ_G8) | (1u64 << SQ_H7));
    }
}
