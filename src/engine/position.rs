use super::board::{BitBoard, Square};
use super::moves::{Move, MoveFlags, do_move, undo_move};
use super::piece::{Color, Piece};
use super::utils;

mod internal;

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub castling: u8,
}

pub struct Position {
    /// Data used to serialize/deserialize FEN.
    pub bitboards: [BitBoard; Piece::COUNT],

    pub side_to_move: Color,
    pub castling: u8,
    pub ep_sq: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,

    /// Data can be computed from the FEN state.
    pub occupancies: [BitBoard; 3],
    pub attack_map: [BitBoard; Color::COUNT],

    /// @TODO: remove undo/redo stack out of Postion,
    /// so position is stateless.
    undo_stack: Vec<(Move, Snapshot)>,
    redo_stack: Vec<(Move, Snapshot)>,
}

impl Position {
    pub fn new() -> Self {
        let bitboards = [
            BitBoard::from(0x000000000000FF00), // White Pawns
            BitBoard::from(0x0000000000000042), // White Knights
            BitBoard::from(0x0000000000000024), // White Bishops
            BitBoard::from(0x0000000000000081), // White Rooks
            BitBoard::from(0x0000000000000008), // White Queens
            BitBoard::from(0x0000000000000010), // White King
            BitBoard::from(0x00FF000000000000), // Black Pawns
            BitBoard::from(0x4200000000000000), // Black Knights
            BitBoard::from(0x2400000000000000), // Black Bishops
            BitBoard::from(0x8100000000000000), // Black Rooks
            BitBoard::from(0x0800000000000000), // Black Queens
            BitBoard::from(0x1000000000000000), // Black King
        ];

        let occupancies = utils::calc_occupancies(&bitboards);
        let attack_map = [BitBoard::from(0x0000000000FF0000), BitBoard::from(0x0000FF0000000000)];

        Self {
            bitboards,
            side_to_move: Color::WHITE,
            castling: MoveFlags::KQkq,
            ep_sq: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            occupancies,
            attack_map,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_parts(
        piece_placement: &str,
        side_to_move: &str,
        castling_rights: &str,
        en_passant_target: &str,
        halfmove_clock: &str,
        fullmove_number: &str,
    ) -> Result<Self, &'static str> {
        let bitboards = utils::parse_board(piece_placement)?;
        let side_to_move = match Color::parse(side_to_move) {
            Some(color) => color,
            None => return Err("Invalid side to move in FEN"),
        };
        let castling = utils::parse_castling(castling_rights)?;
        let halfmove_clock = utils::parse_halfmove_clock(halfmove_clock)?;
        let fullmove_number = utils::parse_fullmove_number(fullmove_number)?;

        let occupancies = utils::calc_occupancies(&bitboards);

        let en_passant = utils::parse_en_passant(en_passant_target)?;

        let mut pos = Self {
            bitboards,
            side_to_move,
            castling,
            ep_sq: en_passant,
            halfmove_clock,
            fullmove_number,
            occupancies,
            attack_map: [BitBoard::new(); 2],
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        pos.attack_map = pos.calc_attack_map();
        Ok(pos)
    }

    pub fn from(fen: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields");
        }

        Self::from_parts(parts[0], parts[1], parts[2], parts[3], parts[4], parts[5])
    }

    pub fn update_cache(&mut self) {
        self.occupancies = utils::calc_occupancies(&self.bitboards);
        self.attack_map = self.calc_attack_map();
    }

    pub fn change_side(&mut self) {
        self.side_to_move = self.side_to_move.opponent()
    }

    pub fn get_piece(&self, sq: Square) -> Piece {
        for i in 0..Piece::COUNT {
            if self.bitboards[i].test(sq.as_u8()) {
                return unsafe { std::mem::transmute(i as u8) };
            }
        }

        Piece::NONE
    }

    pub fn is_legal_move(&mut self, m: &Move) -> bool {
        internal::validate_move(self, &m)
    }

    pub fn legal_move_from_to(&mut self, from_sq: Square, to_sq: Square) -> Option<Move> {
        internal::legal_move_from_to(self, from_sq, to_sq)
    }

    pub fn pseudo_legal_move(&self, sq: Square) -> BitBoard {
        if self.occupancies[self.side_to_move.as_usize()].test(sq.as_u8()) {
            return internal::pseudo_legal_move_from(self, sq);
        }
        BitBoard::new()
    }

    pub fn legal_move(&mut self, sq: Square) -> BitBoard {
        let mut pseudo_legal = self.pseudo_legal_move(sq);

        let mut bits = pseudo_legal.get();

        while bits != 0 {
            let dst_sq = bits.trailing_zeros();

            let m = internal::pseudo_legal_move_from_to(self, sq, Square(dst_sq as u8));
            if !self.is_legal_move(&m) {
                pseudo_legal.unset(dst_sq as u8);
            }

            bits &= bits - 1;
        }

        pseudo_legal
    }

    fn calc_attack_map_impl<const COLOR: u8, const START: u8, const END: u8>(&self) -> BitBoard {
        let mut attack_map = BitBoard::new();

        for i in START..=END {
            // pieces from W to B
            let bb = self.bitboards[i as usize];
            for sq in 0..64 {
                if bb.test(sq) {
                    attack_map |=
                        internal::pseudo_legal_attack_from(self, Square(sq), Color::from(COLOR));
                }
            }
        }

        attack_map
    }

    pub fn calc_attack_map(&self) -> [BitBoard; Color::COUNT] {
        [
            self.calc_attack_map_impl::<{ Color::WHITE.as_u8() }, { Piece::W_START }, { Piece::W_END }>(),
            self.calc_attack_map_impl::<{ Color::BLACK.as_u8() }, { Piece::B_START }, { Piece::B_END }>(),
        ]
    }

    fn gen_snapshot(&self) -> Snapshot {
        Snapshot { castling: self.castling }
    }

    fn restore_snapshot(&mut self, snapshot: Snapshot) {
        self.castling = snapshot.castling;
    }

    // TODO: move UndoRedo to other module
    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.redo_stack.len() > 0
    }

    pub fn do_move(&mut self, m: &Move) -> Snapshot {
        let snapshot = self.gen_snapshot();

        do_move(self, m);

        self.undo_stack.push((m.clone(), snapshot));
        self.redo_stack.clear();

        snapshot
    }

    pub fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }

        let (m, snapshot) = self.undo_stack.pop().unwrap();

        undo_move(self, &m);

        self.redo_stack.push((m, snapshot));
        true
    }

    pub fn redo(&mut self) -> bool {
        if !self.can_redo() {
            return false;
        }

        let (m, snapshot) = self.redo_stack.pop().unwrap();
        // self.restore_snapshot(snapshot);

        do_move(self, &m);

        self.undo_stack.push((m, snapshot));
        true
    }

    /// @TODO: get rid of this method
    pub fn apply_move_str(&mut self, move_str: &str) -> bool {
        match utils::parse_move(move_str) {
            None => false,
            Some((from, to)) => match self.legal_move_from_to(from, to) {
                None => false,
                Some(m) => {
                    do_move(self, &m);
                    true
                }
            },
        }
    }

    // @TODO: move to utils
    pub fn to_string(&self, pad: bool) -> String {
        internal::to_string(self, pad)
    }

    // @TODO: move to utils
    pub fn to_board_string(&self) -> String {
        internal::to_board_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor_new() {
        let pos = Position::new();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::KQkq);
        assert!(pos.ep_sq.is_none());
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_constructor_from_parts() {
        let pos = Position::from_parts(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        )
        .unwrap();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::KQkq);
        assert!(pos.ep_sq.is_none());
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_constructor_from() {
        let pos = Position::from("r1bqk2r/pp1n1ppp/2pbpn2/8/3P4/2N1BN2/PPP2PPP/R2QKB1R w Kq - 6 7")
            .unwrap();

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::K | MoveFlags::q);
        assert!(pos.ep_sq.is_none());
        assert_eq!(pos.halfmove_clock, 6);
        assert_eq!(pos.fullmove_number, 7);
        assert_eq!(
            pos.to_board_string(),
            "r.bqk..rpp.n.ppp..pbpn.............P......N.BN..PPP..PPPR..QKB.R"
        );
    }

    #[test]
    fn test_calc_attack_map() {
        let pos =
            Position::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let attack_maps = pos.calc_attack_map();

        assert_eq!(attack_maps[Color::WHITE.as_usize()].get(), 0x0000000000FF0000);
        assert_eq!(attack_maps[Color::BLACK.as_usize()].get(), 0x0000FF0000000000);
    }

    #[test]
    fn test_trailing_zeros() {
        let mut bits: u64 = 0b10101000;
        let mut count = 0;
        let expect = [3, 5, 7];
        while bits != 0 {
            let tz = bits.trailing_zeros();
            bits &= bits - 1;
            assert_eq!(tz, expect[count]);
            count += 1;
        }
        assert_eq!(count, expect.len());
    }
}
