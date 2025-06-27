use crate::core::move_gen;
use crate::core::zobrist::*;

use super::types::*;

mod internal;
mod utils;

#[derive(Clone, Copy, Debug)]
pub struct SmallSquareList {
    squares: [Option<Square>; 2],
    count: u8,
}

pub type CheckerList = SmallSquareList;

impl CheckerList {
    pub fn new() -> Self {
        Self { squares: [None; 2], count: 0 }
    }

    pub fn count(&self) -> u8 {
        self.count
    }

    pub fn add(&mut self, sq: Square) -> bool {
        if self.count == 2 {
            return false;
        }
        self.squares[self.count as usize] = Some(sq);
        self.count += 1;
        return true;
    }

    pub fn get(&self, index: usize) -> Option<Square> {
        debug_assert!(index < 2, "Index out of bounds for CheckerList: {}", index);
        self.squares[index]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UndoState {
    pub castling_rights: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,

    // @TODO: store more data here if needed
    pub captured_piece: Piece,
    pub occupancies: [BitBoard; 3],
    pub attack_mask: [BitBoard; Color::COUNT],

    pub checkers: [CheckerList; Color::COUNT],

    pub king_squares: [Square; Color::COUNT],
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    /// Data used to serialize/deserialize FEN.
    pub bitboards: [BitBoard; Piece::COUNT],

    pub side_to_move: Color,
    pub state: UndoState,
}

impl Position {
    pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn new() -> Self {
        Self::from_fen(Self::DEFAULT_FEN).unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields");
        }

        let bitboards = utils::parse_board(parts[0])?;
        let side_to_move = match Color::parse(parts[1]) {
            Some(color) => color,
            None => return Err("Invalid side to move in FEN"),
        };
        let castling = utils::parse_castling(parts[2])?;

        let en_passant = utils::parse_en_passant(parts[3]);
        if en_passant.is_none() {
            return Err("Invalid en passant square in FEN");
        }
        let en_passant = en_passant.unwrap();

        let halfmove_clock = utils::parse_halfmove_clock(parts[4])?;
        let fullmove_number = utils::parse_fullmove_number(parts[5])?;

        let state = UndoState {
            castling_rights: castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
            captured_piece: Piece::NONE,
            occupancies: [BitBoard::new(); 3],
            attack_mask: [BitBoard::new(); Color::COUNT],
            checkers: [CheckerList::new(); Color::COUNT],
            king_squares: [Square::NONE; Color::COUNT],
        };

        let mut pos = Position { bitboards, side_to_move, state };
        internal::update_cache(&mut pos);

        Ok(pos)
    }

    pub fn fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            utils::dump_board(&self.bitboards),
            if self.white_to_move() { "w" } else { "b" },
            utils::dump_castling(self.state.castling_rights),
            match self.state.en_passant {
                Some(sq) => sq.to_string(),
                None => "-".to_string(),
            },
            self.state.halfmove_clock,
            self.state.fullmove_number
        )
    }

    pub fn zobrist(&self) -> ZobristHash {
        zobrist_hash(&self)
    }

    pub fn white_to_move(&self) -> bool {
        self.side_to_move == Color::WHITE
    }

    pub fn get_piece_at(&self, sq: Square) -> Piece {
        let sq = sq.as_u8();
        if self.state.occupancies[Color::BOTH.as_usize()].test(sq) == false {
            return Piece::NONE;
        }

        let color = if self.state.occupancies[Color::WHITE.as_usize()].test(sq) {
            Color::WHITE
        } else {
            Color::BLACK
        };

        for i in 0..PieceType::COUNT {
            let piece = Piece::get_piece(color, PieceType(i));
            if self.bitboards[piece.as_usize()].test(sq) {
                return piece;
            }
        }
        panic!("No piece found at square {}", sq);
    }

    pub fn get_color_at(&self, sq: Square) -> Color {
        let is_white = self.state.occupancies[Color::WHITE.as_usize()].test(sq.as_u8());
        let is_black = self.state.occupancies[Color::BLACK.as_usize()].test(sq.as_u8());
        if cfg!(debug_assertions) {
            debug_assert!(is_white ^ is_black, "Square {} has both colors", sq);
            let piece = self.get_piece_at(sq);
            let debug_color = piece.color();
            debug_assert!(
                (is_white && debug_color == Color::WHITE)
                    || (is_black && debug_color == Color::BLACK),
                "Square {} has color {:?}, but piece is {:?}",
                sq,
                debug_color,
                piece
            );
        }

        if !is_white && !is_black {
            debug_assert!(self.state.occupancies[Color::BOTH.as_usize()].test(sq.as_u8()) == false);
            return Color::NONE;
        }

        if is_white { Color::WHITE } else { Color::BLACK }
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        self.state.king_squares[color.as_usize()]
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let checker_count = self.state.checkers[color.as_usize()].count();

        if cfg!(debug_assertions) && checker_count != 0 {
            let king_sq = self.get_king_square(color);
            let attack_map = self.state.attack_mask[color.flip().as_usize()];
            debug_assert!(
                attack_map.test(king_sq.as_u8()),
                "King square {} is not attacked by opponent's pieces",
                king_sq
            );
        }

        checker_count != 0
    }

    pub fn make_move(&mut self, mv: Move) -> (UndoState, bool) {
        internal::make_move(self, mv)
    }

    pub fn unmake_move(&mut self, mv: Move, undo_state: &UndoState) {
        internal::unmake_move(self, mv, undo_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor_from_parts() {
        const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from_fen(FEN).unwrap();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.state.castling_rights, CastlingRight::KQkq);
        assert!(pos.state.en_passant.is_none());
        assert_eq!(pos.state.halfmove_clock, 0);
        assert_eq!(pos.state.fullmove_number, 1);
        assert_eq!(pos.fen(), FEN);
    }

    #[test]
    fn test_constructor_from() {
        const FEN: &str = "r1bqk2r/pp1n1ppp/2pbpn2/8/3P4/2N1BN2/PPP2PPP/R2QKB1R w Kq - 6 7";
        let pos = Position::from_fen(FEN).unwrap();

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.state.castling_rights, CastlingRight::K | CastlingRight::q);
        assert!(pos.state.en_passant.is_none());
        assert_eq!(pos.state.halfmove_clock, 6);
        assert_eq!(pos.state.fullmove_number, 7);
        assert_eq!(pos.fen(), FEN);

        assert_eq!(pos.get_king_square(Color::WHITE), Square::E1);
        assert_eq!(pos.get_king_square(Color::BLACK), Square::E8);
    }

    #[test]
    fn test_checkers() {
        let pos = Position::from_fen("r3k3/8/4B3/8/4r3/8/2n5/R3K2R w - - 0 1").unwrap();

        let checkers = pos.state.checkers[Color::WHITE.as_usize()];
        assert_eq!(checkers.count(), 2);
        let sq1 = checkers.get(0).unwrap();
        let sq2 = checkers.get(1).unwrap();
        assert!(
            matches!((sq1, sq2), (Square::C2, Square::E4) | (Square::E4, Square::C2)),
            "Checkers should be at C2 and E4, got {:?} and {:?}",
            sq1,
            sq2
        );
    }

    #[test]
    fn test_full_move_number() {
        let mut pos = Position::new();
        assert_eq!(pos.state.fullmove_number, 1);

        pos.make_move(Move::new(Square::E2, Square::E4, MoveType::Normal, None));
        assert_eq!(pos.state.fullmove_number, 1);

        pos.make_move(Move::new(Square::E7, Square::E5, MoveType::Normal, None));
        assert_eq!(pos.state.fullmove_number, 2);
    }

    const UNDO_TEST_FEN: &str = "4k2r/1p6/8/P7/8/8/2p5/4K3 b k - 0 10";

    #[test]
    fn undo_castling_should_put_rook_back() {
        let mut pos = Position::from_fen(UNDO_TEST_FEN).unwrap();
        let mv = Move::new(Square::E8, Square::G8, MoveType::Castling, None);

        let undo_state = pos.make_move(mv).0;

        assert_eq!(pos.get_piece_at(Square::G8), Piece::B_KING);
        assert_eq!(pos.get_piece_at(Square::F8), Piece::B_ROOK);
        pos.unmake_move(mv, &undo_state);
        assert_eq!(pos.get_piece_at(Square::E8), Piece::B_KING);
        assert_eq!(pos.get_piece_at(Square::H8), Piece::B_ROOK);
    }

    #[test]
    fn undo_en_passant_should_put_pawn_back() {
        let mut pos = Position::from_fen(UNDO_TEST_FEN).unwrap();
        let mv = Move::new(Square::B7, Square::B5, MoveType::Normal, None);
        pos.make_move(mv);

        let mv = Move::new(Square::A5, Square::B6, MoveType::EnPassant, None);
        let undo_state = pos.make_move(mv).0;

        assert_eq!(pos.get_piece_at(Square::B6), Piece::W_PAWN);
        assert_eq!(pos.get_piece_at(Square::A5), Piece::NONE);
        assert_eq!(pos.get_piece_at(Square::B5), Piece::NONE);

        pos.unmake_move(mv, &undo_state);

        assert_eq!(pos.get_piece_at(Square::A5), Piece::W_PAWN);
        assert_eq!(pos.get_piece_at(Square::B5), Piece::B_PAWN);
    }

    #[test]
    fn undo_should_revert_promoted_piece() {
        let mut pos = Position::from_fen(UNDO_TEST_FEN).unwrap();
        let mv = Move::new(Square::C2, Square::C1, MoveType::Promotion, Some(PieceType::BISHOP));
        let undo_state = pos.make_move(mv).0;

        assert_eq!(pos.get_piece_at(Square::C2), Piece::NONE);
        assert_eq!(pos.get_piece_at(Square::C1), Piece::B_BISHOP);

        pos.unmake_move(mv, &undo_state);

        assert_eq!(pos.get_piece_at(Square::C2), Piece::B_PAWN);
        assert_eq!(pos.get_piece_at(Square::C1), Piece::NONE);
    }
}
