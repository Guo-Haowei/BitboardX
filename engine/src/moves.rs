use crate::board::Board;
use crate::types::*;

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

    Some((
        make_square(from_file, from_rank),
        make_square(to_file, to_rank),
    ))
}

fn move_pawn<const IS_WHITE: bool>(board: &Board, file: u8, rank: u8) -> u64 {
    if IS_WHITE && rank == RANK_7 || !IS_WHITE && rank == RANK_2 {
        panic!("TODO: Handle promotion");
    }

    let bitboard = 1u64 << make_square(file, rank);

    let mut moves = 0u64;

    let new_pos_1 = if IS_WHITE {
        bitboard << 8
    } else {
        bitboard >> 8
    };
    if new_pos_1 & board.occupancies[Color::Both as usize] == 0 {
        moves |= new_pos_1;
    }

    if (IS_WHITE && rank == RANK_2 || !IS_WHITE && rank == RANK_7) && moves != 0 {
        let new_pos_2 = if IS_WHITE {
            bitboard << 16
        } else {
            bitboard >> 16
        };
        if new_pos_2 & board.occupancies[Color::Both as usize] == 0 {
            moves |= new_pos_2;
        }
    }

    moves
}

pub fn gen_moves(board: &Board, file: u8, rank: u8) -> u64 {
    let square = make_square(file, rank);
    let mask = 1u64 << square;

    for i in 0..board.bitboards.len() {
        if (board.bitboards[i] & mask) == 0 {
            continue;
        }

        let piece: Piece = unsafe { std::mem::transmute(i as u8) };
        let color = if piece <= Piece::WhiteKing {
            Color::White
        } else {
            Color::Black
        };
        if color != board.side_to_move {
            return 0;
        }

        return match piece {
            Piece::WhitePawn => move_pawn::<true>(board, file, rank),
            Piece::BlackPawn => move_pawn::<false>(board, file, rank),
            _ => 0,
            // Piece::WhiteKnight => 'N',
            // Piece::WhiteBishop => 'B',
            // Piece::WhiteRook => 'R',
            // Piece::WhiteQueen => 'Q',
            // Piece::WhiteKing => 'K',
            // Piece::BlackPawn => 'p',
            // Piece::BlackKnight => 'n',
            // Piece::BlackBishop => 'b',
            // Piece::BlackRook => 'r',
            // Piece::BlackQueen => 'q',
            // Piece::BlackKing => 'k',
            // Piece::Count => '.',
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

        let moves = gen_moves(&board, FILE_E, RANK_2);
        assert_eq!(moves, (1u64 << SQ_E3) | (1u64 << SQ_E4));
    }

    #[test]
    fn move_unmoved_black_pawn() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, FILE_D, RANK_7);
        assert_eq!(moves, (1u64 << SQ_D6) | (1u64 << SQ_D5));
    }
}
