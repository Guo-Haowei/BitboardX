use crate::engine::move_gen;

use super::types::*;
use super::utils::parse_move;

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
        assert!(index < 2, "Index out of bounds for CheckerList: {}", index);
        self.squares[index]
    }
}

#[derive(Clone, Copy)]
pub struct UndoState {
    pub castling: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub dst_piece: Piece,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    /// Data used to serialize/deserialize FEN.
    pub bitboards: [BitBoard; Piece::COUNT],

    pub side_to_move: Color,
    pub castling: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,

    /// Data can be computed from the FEN state.
    pub occupancies: [BitBoard; 3],
    pub attack_mask: [BitBoard; Color::COUNT],
    pub pin_map: [BitBoard; Color::COUNT],
    pub checkers: [CheckerList; Color::COUNT],
}

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Position {
    pub fn new() -> Self {
        Self::from(DEFAULT_FEN).unwrap()
    }

    pub fn from(fen: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields");
        }

        let bitboards = utils::parse_board(parts[0])?;
        let side_to_move = match Color::parse(parts[1]) {
            Some(color) => color.opponent(),
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

        let mut pos = Self {
            bitboards,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
            occupancies: [BitBoard::new(); 3],
            attack_mask: [BitBoard::new(); Color::COUNT],
            pin_map: [BitBoard::new(); Color::COUNT],
            checkers: [CheckerList::new(); Color::COUNT],
        };

        pos.post_move();
        Ok(pos)
    }

    pub fn fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            utils::dump_board(&self.bitboards),
            if self.side_to_move == Color::WHITE { "w" } else { "b" },
            utils::dump_castling(self.castling),
            match self.en_passant {
                Some(sq) => sq.to_string(),
                None => "-".to_string(),
            },
            self.halfmove_clock,
            self.fullmove_number
        )
    }

    // @TODO: make private
    pub fn post_move(&mut self) {
        self.side_to_move = self.side_to_move.opponent();

        // update occupancies
        self.occupancies[Color::WHITE.as_usize()] = self.bitboards[Piece::W_PAWN.as_usize()]
            | self.bitboards[Piece::W_KNIGHT.as_usize()]
            | self.bitboards[Piece::W_BISHOP.as_usize()]
            | self.bitboards[Piece::W_ROOK.as_usize()]
            | self.bitboards[Piece::W_QUEEN.as_usize()]
            | self.bitboards[Piece::W_KING.as_usize()];
        self.occupancies[Color::BLACK.as_usize()] = self.bitboards[Piece::B_PAWN.as_usize()]
            | self.bitboards[Piece::B_KNIGHT.as_usize()]
            | self.bitboards[Piece::B_BISHOP.as_usize()]
            | self.bitboards[Piece::B_ROOK.as_usize()]
            | self.bitboards[Piece::B_QUEEN.as_usize()]
            | self.bitboards[Piece::B_KING.as_usize()];
        self.occupancies[Color::BOTH.as_usize()] =
            self.occupancies[Color::WHITE.as_usize()] | self.occupancies[Color::BLACK.as_usize()];

        // update attack maps
        self.update_attack_map_and_checker();

        // maybe only need to update the side to move attack map?
        self.pin_map[Color::WHITE.as_usize()] = move_gen::generate_pin_map(self, Color::WHITE);
        self.pin_map[Color::BLACK.as_usize()] = move_gen::generate_pin_map(self, Color::BLACK);
    }

    pub fn get_piece_at(&self, sq: Square) -> Piece {
        for i in 0..Piece::COUNT {
            if self.bitboards[i].test(sq.as_u8()) {
                return unsafe { std::mem::transmute(i as u8) };
            }
        }

        Piece::NONE
    }

    pub fn get_color_at(&self, sq: Square) -> Color {
        let is_white = self.occupancies[Color::WHITE.as_usize()].test(sq.as_u8());
        let is_black = self.occupancies[Color::BLACK.as_usize()].test(sq.as_u8());
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
            assert!(self.occupancies[Color::BOTH.as_usize()].test(sq.as_u8()) == false);
            return Color::NONE;
        }

        if is_white { Color::WHITE } else { Color::BLACK }
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        let piece = Piece::get_piece(color, PieceType::King);
        let bb = self.bitboards[piece.as_usize()];
        debug_assert!(bb.any(), "No king found for color {:?}", color);
        Square(bb.get().trailing_zeros() as u8)
    }

    pub fn is_square_pinned(&self, sq: Square, color: Color) -> bool {
        let pin_map = &self.pin_map[color.as_usize()];
        pin_map.test(sq.as_u8())
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let checker_count = self.checkers[color.as_usize()].count();

        if cfg!(debug_assertions) && checker_count != 0 {
            let king_sq = self.get_king_square(color);
            let attack_map = self.attack_mask[color.opponent().as_usize()];
            debug_assert!(
                attack_map.test(king_sq.as_u8()),
                "King square {} is not attacked by opponent's pieces",
                king_sq
            );
        }

        checker_count != 0
    }

    fn update_attack_map_and_checker(&mut self) {
        let mut checkers: [CheckerList; Color::COUNT] = [CheckerList::new(); Color::COUNT];

        for color in [Color::WHITE, Color::BLACK] {
            let mut attack_mask = BitBoard::new();
            let opponent = color.opponent();
            let king_sq = self.get_king_square(opponent);
            for i in 0..PieceType::None as u8 {
                let piece_type = unsafe { std::mem::transmute::<u8, PieceType>(i as u8) };
                let piece = Piece::get_piece(color, piece_type);
                attack_mask |= move_gen::calc_attack_map_impl(
                    self,
                    piece,
                    king_sq,
                    &mut checkers[opponent.as_usize()],
                );
            }
            self.attack_mask[color.as_usize()] = attack_mask;
        }

        self.checkers = checkers;
    }

    pub fn restore(&mut self, undo_state: &UndoState) {
        self.castling = undo_state.castling;
        self.en_passant = undo_state.en_passant;

        self.halfmove_clock = undo_state.halfmove_clock;
        self.fullmove_number = undo_state.fullmove_number;
    }

    pub fn make_move(&mut self, m: Move) -> UndoState {
        internal::make_move(self, m)
    }

    pub fn unmake_move(&mut self, m: Move, undo_state: &UndoState) {
        internal::unmake_move(self, m, undo_state)
    }

    /// @TODO: get rid of this method
    pub fn apply_move_str(&mut self, move_str: &str) -> bool {
        let m = parse_move(move_str);
        if m.is_none() {
            return false;
        }

        let (from, to, promotion) = m.unwrap();

        let legal_moves = move_gen::legal_moves(&self);
        for m in legal_moves.iter() {
            if m.from_sq() == from && m.to_sq() == to && m.get_promotion() == promotion {
                self.make_move(m.clone());
                return true;
            }
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor_from_parts() {
        const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(FEN).unwrap();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, CastlingRight::KQkq);
        assert!(pos.en_passant.is_none());
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert_eq!(pos.fen(), FEN);
    }

    #[test]
    fn test_constructor_from() {
        const FEN: &str = "r1bqk2r/pp1n1ppp/2pbpn2/8/3P4/2N1BN2/PPP2PPP/R2QKB1R w Kq - 6 7";
        let pos = Position::from(FEN).unwrap();

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, CastlingRight::K | CastlingRight::q);
        assert!(pos.en_passant.is_none());
        assert_eq!(pos.halfmove_clock, 6);
        assert_eq!(pos.fullmove_number, 7);
        assert_eq!(pos.fen(), FEN);

        assert_eq!(pos.get_king_square(Color::WHITE), Square::E1);
        assert_eq!(pos.get_king_square(Color::BLACK), Square::E8);
    }

    #[test]
    fn test_checkers() {
        let pos = Position::from("r3k3/8/4B3/8/4r3/8/2n5/R3K2R w - - 0 1").unwrap();

        let checkers = pos.checkers[Color::WHITE.as_usize()];
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
    fn test_is_square_pinned() {
        // 2 . . . . . . . k
        // 1 K B . . . . . r
        //   a b c d e f g h
        let pos = Position::from("8/8/8/8/8/8/7k/KB5r w - - 0 1").unwrap();

        assert!(pos.is_square_pinned(Square::B1, Color::WHITE));
    }

    #[test]
    fn test_rook_pin_pawn() {
        let pos = Position::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

        let is_pinned = pos.is_square_pinned(Square::B5, Color::WHITE);

        assert!(is_pinned, "Pawn B5 is pinned by rook on H5");
    }

    #[test]
    fn test_full_move_number() {
        let mut pos = Position::new();
        assert_eq!(pos.fullmove_number, 1);

        pos.make_move(Move::new(Square::E2, Square::E4, MoveType::Normal, None));
        assert_eq!(pos.fullmove_number, 1);

        pos.make_move(Move::new(Square::E7, Square::E5, MoveType::Normal, None));
        assert_eq!(pos.fullmove_number, 2);
    }
}
