use crate::board::Board;
use crate::types::*;

const FILE_A_BOUND: u64 = 0x0101010101010101;
const FILE_H_BOUND: u64 = 0x8080808080808080;
const RANK_1_BOUND: u64 = 0x00000000000000FF;
const RANK_8_BOUND: u64 = 0xFF00000000000000;

fn shift_east(bb: u64) -> u64 {
    (bb & !FILE_H_BOUND) << 1
}

fn shift_west(bb: u64) -> u64 {
    (bb & !FILE_A_BOUND) >> 1
}

fn shift_north(bb: u64) -> u64 {
    (bb & !RANK_8_BOUND) << 8
}

fn shift_south(bb: u64) -> u64 {
    (bb & !RANK_1_BOUND) >> 8
}

fn shift_north_east(bb: u64) -> u64 {
    (bb & !FILE_H_BOUND & !RANK_8_BOUND) << 9
}

fn shift_north_west(bb: u64) -> u64 {
    (bb & !FILE_A_BOUND & !RANK_8_BOUND) << 7
}

fn shift_south_east(bb: u64) -> u64 {
    (bb & !FILE_H_BOUND & !RANK_1_BOUND) >> 7
}

fn shift_south_west(bb: u64) -> u64 {
    (bb & !FILE_A_BOUND & !RANK_1_BOUND) >> 9
}

const SHIFT_FUNCS: [fn(u64) -> u64; 8] = [
    shift_north,
    shift_south,
    shift_east,
    shift_west,
    shift_north_east,
    shift_north_west,
    shift_south_east,
    shift_south_west,
];

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
    let bitboard = 1u64 << make_square(file, rank);
    let opposite_color = get_opposite_color(color);

    for i in START..END {
        let mut new_pos = SHIFT_FUNCS[i as usize](bitboard);

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

fn move_pawn<const IS_WHITE: bool>(board: &Board, file: u8, rank: u8) -> u64 {
    let bitboard = 1u64 << make_square(file, rank);
    let mut moves = 0u64;
    let opposite_color = if IS_WHITE { Color::Black } else { Color::White };

    // Promotion
    if IS_WHITE && rank == RANK_7 || !IS_WHITE && rank == RANK_2 {
        panic!("TODO: Handle promotion");
    }

    // Move forward
    let new_pos_1 = if IS_WHITE { shift_north(bitboard) } else { shift_south(bitboard) };

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
    let attack_left = if IS_WHITE { shift_north_west(bitboard) } else { shift_south_west(bitboard) };
    if attack_left & board.occupancies[opposite_color as usize] != 0 {
        moves |= attack_left;
    }
    let attack_right = if IS_WHITE { shift_north_east(bitboard) } else { shift_south_east(bitboard) };
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
            _ => 0,
        };
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_unmoved_white_pawn() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_E2);
        assert_eq!(moves, (1u64 << SQ_E3) | (1u64 << SQ_E4));
    }

    #[test]
    fn move_unmoved_black_pawn() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, SQ_D7);
        assert_eq!(moves, (1u64 << SQ_D6) | (1u64 << SQ_D5));
    }
}
