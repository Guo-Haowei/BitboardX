use crate::types::*;
use crate::board::Board;

fn move_pawn<const IS_WHITE: bool>(board: &Board, file: File, rank: Rank) -> u64 {
    if IS_WHITE && rank == Rank::R7 || !IS_WHITE && rank == Rank::R2 {
        panic!("TODO: Handle promotion");
    }

    let bitboard = 1u64 << ((rank as u8) * 8 + file as u8);

    let mut moves = 0u64;

    let new_pos_1 = if IS_WHITE { bitboard << 8 } else { bitboard >> 8 };
    if new_pos_1 & board.occupancies[Color::Both as usize] == 0 {
        moves |= new_pos_1;
    }

    if (IS_WHITE && rank == Rank::R2 || !IS_WHITE && rank == Rank::R7) && moves != 0 {
        let new_pos_2 = if IS_WHITE { bitboard << 16 } else { bitboard >> 16 };
        if new_pos_2 & board.occupancies[Color::Both as usize] == 0 {
            moves |= new_pos_2;
        }
    }

    moves
}

pub fn gen_moves(board: &Board, file: File, rank: Rank) -> u64 {
    let square = (rank as u8) * 8 + file as u8;
    let mask = 1u64 << square;

    for i in 0..board.bitboards.len() {
        if (board.bitboards[i] & mask) == 0 {
            continue;
        }

        let piece : Piece = unsafe { std::mem::transmute(i as u8) };
        let color = if piece <= Piece::WhiteKing { Color::White } else { Color::Black };
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

        let moves = gen_moves(&board, File::E, Rank::R2);
        assert_eq!(moves, 1u64 << (Square::E3 as u8) | 1u64 << (Square::E4 as u8));
    }

    #[test]
    fn move_unmoved_black_pawn() {
        let mut board = Board::new();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        assert!(board.parse_fen(fen).is_ok());

        let moves = gen_moves(&board, File::D, Rank::R7);
        assert_eq!(moves, 1u64 << (Square::D6 as u8) | 1u64 << (Square::D5 as u8));
    }
}
