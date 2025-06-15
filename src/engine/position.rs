use crate::engine::move_gen;

use super::board::*;
use super::types::*;
use super::utils;

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
pub struct Snapshot {
    pub castling: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub to_piece: Piece,
}

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

        // @NOTE: this is a bit hacky, to set the side to move to opposite color
        // because post_move() will change the side to move
        let side_to_move = Color::BLACK;

        let mut result = Self {
            bitboards,
            side_to_move,
            castling: MoveFlags::KQkq,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            occupancies: [BitBoard::new(); 3],
            attack_mask: [BitBoard::new(); Color::COUNT],
            pin_map: [BitBoard::new(); Color::COUNT],
            checkers: [CheckerList::new(); Color::COUNT],
            // @TODO: refactor
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        result.post_move();
        result
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

        let en_passant = utils::parse_en_passant(parts[3])?;

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
            // @TODO: move away
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        pos.post_move();
        Ok(pos)
    }

    // @TODO: make private
    pub fn post_move(&mut self) {
        self.side_to_move = self.side_to_move.opponent();
        self.occupancies = utils::calc_occupancies(&self.bitboards);
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
        if !self.occupancies[Color::BOTH.as_usize()].test(sq.as_u8()) {
            return Color::NONE;
        }

        let is_white = self.occupancies[Color::WHITE.as_usize()].test(sq.as_u8());
        if cfg!(debug_assertions) {
            let is_black = self.occupancies[Color::BLACK.as_usize()].test(sq.as_u8());
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

        if is_white { Color::WHITE } else { Color::BLACK }
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        let piece = Piece::get_piece(color, PieceType::King);
        let mut bb = self.bitboards[piece.as_usize()];
        debug_assert!(bb.any(), "No king found for color {:?}", color);
        let sq = bb.first_nonzero_sq();
        if cfg!(debug_assertions) {
            bb.remove_first_nonzero_sq();
            debug_assert!(bb.none(), "only one king should be on the board");
        }
        sq
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

    // @TODO: remove these methods, call move_gen directly

    pub fn is_move_legal(&mut self, m: &Move) -> bool {
        move_gen::is_pseudo_move_legal(self, &m)
    }

    pub fn pseudo_legal_moves(&self) -> MoveList {
        move_gen::pseudo_legal_moves(self)
    }

    pub fn legal_moves(&mut self) -> MoveList {
        move_gen::legal_moves(self)
    }

    pub fn update_attack_map_and_checker(&mut self) {
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

    pub fn restore(&mut self, snapshot: &Snapshot) {
        self.castling = snapshot.castling;
        self.en_passant = snapshot.en_passant;

        self.halfmove_clock = snapshot.halfmove_clock;
        self.fullmove_number = snapshot.fullmove_number;
    }

    pub fn make_move(&mut self, m: Move) -> Snapshot {
        // @TODO: refactor this code, pretty please

        let castling = self.castling;
        let en_passant = self.en_passant;
        let halfmove_clock = self.halfmove_clock;
        let fullmove_number = self.fullmove_number;

        let disabled_castling = drop_castling(
            self,
            m.from_sq(),
            m.to_sq(),
            self.get_piece_at(m.from_sq()),
            self.get_piece_at(m.to_sq()),
        );

        let from = self.get_piece_at(m.from_sq());
        do_move_ep(self, m, from);

        debug_assert!(self.occupancies[self.side_to_move.as_usize()].test(m.from_sq().as_u8()));

        let from_sq = m.from_sq();
        let to_sq = m.to_sq();

        let to_piece = self.get_piece_at(to_sq);

        move_piece(&mut self.bitboards[from.as_usize()], from_sq, to_sq);

        if to_piece != Piece::NONE {
            self.bitboards[to_piece.as_usize()].unset(m.to_sq().as_u8()); // Clear the 'to' square for the captured piece
        }

        do_castling(self, m, from);

        do_promotion(self, m, from);

        self.post_move();

        self.castling &= !disabled_castling;
        self.en_passant = update_en_passant_square(self, m.from_sq(), m.to_sq(), from);

        Snapshot { castling, en_passant, halfmove_clock, fullmove_number, to_piece }
    }

    pub fn unmake_move(&mut self, m: Move, snapshot: &Snapshot) {
        let from = self.get_piece_at(m.to_sq());

        undo_move_generic(self, m, from, snapshot.to_piece);

        undo_promotion(self, m);

        undo_move_ep(self, m, from);

        undo_castling(self, m, from);

        self.post_move();

        self.restore(snapshot);
    }

    /// @TODO: get rid of this method
    pub fn apply_move_str(&mut self, move_str: &str) -> bool {
        let m = utils::parse_move(move_str);
        if m.is_none() {
            return false;
        }

        let (from, to) = m.unwrap();

        let legal_moves = self.legal_moves();
        for m in legal_moves.iter() {
            if m.from_sq() == from && m.to_sq() == to {
                self.make_move(m.clone());
                return true;
            }
        }

        return false;
    }

    // TODO: move UndoRedo to other module
    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.redo_stack.len() > 0
    }

    pub fn do_move(&mut self, m: Move) -> Snapshot {
        let snapshot = self.make_move(m);

        self.undo_stack.push((m.clone(), snapshot));
        self.redo_stack.clear();

        snapshot
    }

    pub fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }

        let (m, snapshot) = self.undo_stack.pop().unwrap();

        self.unmake_move(m, &snapshot);

        self.redo_stack.push((m, snapshot));
        true
    }

    pub fn redo(&mut self) -> bool {
        if !self.can_redo() {
            return false;
        }

        let (m, snapshot) = self.redo_stack.pop().unwrap();
        // self.restore_snapshot(snapshot);

        self.make_move(m);

        self.undo_stack.push((m, snapshot));
        true
    }
    // @TODO: move to utils
    pub fn to_string(&self, pad: bool) -> String {
        utils::to_string(self, pad)
    }

    // @TODO: move to utils
    pub fn to_board_string(&self) -> String {
        utils::to_board_string(self)
    }
}

////////////////////////////
////////////////////////////
fn do_move_ep(pos: &mut Position, m: Move, from: Piece) {
    let (to_file, _) = m.to_sq().file_rank();

    // check if it's an en passant capture
    if m.get_type() == MoveType::EnPassant {
        // capture the opponent's pawn passed en passant square
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(from.color().opponent(), PieceType::Pawn);

        debug_assert!(pos.get_piece_at(enemy_sq) == enemy);
        debug_assert!(pos.get_piece_at(m.to_sq()) == Piece::NONE);

        // Remove the captured pawn from the board
        pos.bitboards[enemy.as_usize()].unset(enemy_sq.as_u8());
    }
}

fn undo_move_ep(pos: &mut Position, m: Move, from: Piece) {
    if m.get_type() == MoveType::EnPassant {
        // Restore the captured pawn on the en passant square
        let (to_file, _) = m.to_sq().file_rank();
        let (_, from_rank) = m.from_sq().file_rank();
        let enemy_sq = Square::make(to_file, from_rank);
        let enemy = Piece::get_piece(from.color().opponent(), PieceType::Pawn);

        debug_assert!(pos.get_piece_at(enemy_sq) == Piece::NONE);

        // Place the captured pawn back on the board
        pos.bitboards[enemy.as_usize()].set(enemy_sq.as_u8());
    }
}

fn move_piece(board: &mut BitBoard, from_sq: Square, to_sq: Square) {
    debug_assert!(board.test(from_sq.as_u8()), "No piece found on 'from' square");
    board.unset(from_sq.as_u8());
    board.set(to_sq.as_u8());
}

fn undo_move_generic(pos: &mut Position, m: Move, from: Piece, to: Piece) {
    let from_sq = m.from_sq();
    let to_sq = m.to_sq();

    move_piece(&mut pos.bitboards[from.as_usize()], to_sq, from_sq);

    if to != Piece::NONE {
        pos.bitboards[to.as_usize()].set(m.to_sq().as_u8()); // Place captured piece back on 'to' square
    }
}

const CASTLING_ROOK_SQUARES: [(Piece, Square, Square); 4] = [
    (Piece::W_ROOK, Square::H1, Square::F1), // White King-side
    (Piece::W_ROOK, Square::A1, Square::D1), // White Queen-side
    (Piece::B_ROOK, Square::H8, Square::F8), // Black King-side
    (Piece::B_ROOK, Square::A8, Square::D8), // Black Queen-side
];

fn castling_type(from: Piece, from_sq: Square, to_sq: Square) -> Castling {
    match (from, from_sq, to_sq) {
        (Piece::W_KING, Square::E1, Square::G1) => Castling::WhiteKingSide,
        (Piece::W_KING, Square::E1, Square::C1) => Castling::WhiteQueenSide,
        (Piece::B_KING, Square::E8, Square::G8) => Castling::BlackKingSide,
        (Piece::B_KING, Square::E8, Square::C8) => Castling::BlackQueenSide,
        _ => Castling::None,
    }
}

fn do_promotion(pos: &mut Position, m: Move, from: Piece) {
    if m.get_type() != MoveType::Promotion {
        return;
    }

    assert!(
        from.get_type() == PieceType::Pawn,
        "Promotion must be from a pawm, got '{}'",
        from.to_char()
    );

    let color = from.color();
    let to_sq = m.to_sq();
    let promotion = Piece::get_piece(color, m.get_promotion().unwrap());

    pos.bitboards[from.as_usize()].unset(to_sq.as_u8()); // Remove the pawn from the board
    pos.bitboards[promotion.as_usize()].set(to_sq.as_u8()); // Place the promoted piece on the board
}

fn undo_promotion(pos: &mut Position, m: Move) {
    if m.get_type() != MoveType::Promotion {
        return;
    }

    // from square is the square of the promoted piece
    let from_sq = m.from_sq();
    let piece = pos.get_piece_at(from_sq);
    let color = piece.color();
    let promotion = Piece::get_piece(color, m.get_promotion().unwrap());
    let pawn = Piece::get_piece(color, PieceType::Pawn);

    pos.bitboards[pawn.as_usize()].set(from_sq.as_u8()); // Place the pawn back on the board
    pos.bitboards[promotion.as_usize()].unset(from_sq.as_u8()); // Remove the promoted piece from the board
}

fn do_castling(pos: &mut Position, m: Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }
    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], from_sq, to_sq);
}

fn undo_castling(pos: &mut Position, m: Move, from: Piece) {
    // Restore Rook position
    let index = castling_type(from, m.from_sq(), m.to_sq());
    if index == Castling::None {
        return;
    }

    let (piece, from_sq, to_sq) = CASTLING_ROOK_SQUARES[index as usize];
    move_piece(&mut pos.bitboards[piece.as_usize()], to_sq, from_sq);
}

// if castling rights are already disabled, return
// if king moved, disable castling rights, return
// if rook moved, disable castling rights, return
// if rook taken out, disable castling rights, return
fn drop_castling(pos: &Position, from_sq: Square, to_sq: Square, from: Piece, to: Piece) -> u8 {
    fn helper<const BIT: u8>(
        pos: &Position,
        from_sq: Square,
        to_sq: Square,
        from: Piece,
        to: Piece,
    ) -> u8 {
        if pos.castling & BIT == 0 {
            return 0;
        }

        if from == Piece::W_KING && (BIT & MoveFlags::KQ) != 0 {
            return BIT;
        }

        if from == Piece::B_KING && (BIT & MoveFlags::kq) != 0 {
            return BIT;
        }

        match (from, from_sq) {
            (Piece::W_ROOK, Square::A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
            (Piece::W_ROOK, Square::H1) if (BIT & MoveFlags::K) != 0 => return BIT,
            (Piece::B_ROOK, Square::A8) if (BIT & MoveFlags::q) != 0 => return BIT,
            (Piece::B_ROOK, Square::H8) if (BIT & MoveFlags::k) != 0 => return BIT,
            _ => {}
        }

        match (to, to_sq) {
            (Piece::W_ROOK, Square::A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
            (Piece::W_ROOK, Square::H1) if (BIT & MoveFlags::K) != 0 => return BIT,
            (Piece::B_ROOK, Square::A8) if (BIT & MoveFlags::q) != 0 => return BIT,
            (Piece::B_ROOK, Square::H8) if (BIT & MoveFlags::k) != 0 => return BIT,
            _ => {}
        }

        0
    }

    let mut drop_castling = 0;
    drop_castling |= helper::<{ MoveFlags::K }>(pos, from_sq, to_sq, from, to);
    drop_castling |= helper::<{ MoveFlags::Q }>(pos, from_sq, to_sq, from, to);
    drop_castling |= helper::<{ MoveFlags::k }>(pos, from_sq, to_sq, from, to);
    drop_castling |= helper::<{ MoveFlags::q }>(pos, from_sq, to_sq, from, to);
    drop_castling
}

fn update_en_passant_square(
    pos: &Position,
    from_sq: Square,
    to_sq: Square,
    from: Piece,
) -> Option<Square> {
    if from.get_type() != PieceType::Pawn {
        return None;
    }

    let (file, from_rank) = from_sq.file_rank();
    let (_file, to_rank) = to_sq.file_rank();

    if match (from, from_rank, to_rank) {
        (Piece::W_PAWN, RANK_2, RANK_4) => true,
        (Piece::B_PAWN, RANK_7, RANK_5) => true,
        _ => false,
    } {
        assert_eq!(file, _file);
        // check if there's opponent's pawn on the left or right of 'to' square
        let board = &pos.bitboards[if from == Piece::W_PAWN {
            Piece::B_PAWN.as_usize()
        } else {
            Piece::W_PAWN.as_usize()
        }];

        if (file < FILE_H && board.test(Square::make(file + 1, to_rank).as_u8()))
            || (file > FILE_A && board.test(Square::make(file - 1, to_rank).as_u8()))
        {
            return Some(Square::make(file, (from_rank + to_rank) / 2));
        }
    }

    None
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
        assert!(pos.en_passant.is_none());
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_constructor_from_parts() {
        let pos =
            Position::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::KQkq);
        assert!(pos.en_passant.is_none());
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
        assert!(pos.en_passant.is_none());
        assert_eq!(pos.halfmove_clock, 6);
        assert_eq!(pos.fullmove_number, 7);
        assert_eq!(
            pos.to_board_string(),
            "r.bqk..rpp.n.ppp..pbpn.............P......N.BN..PPP..PPPR..QKB.R"
        );

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
}
