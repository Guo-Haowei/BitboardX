use super::position::Position;
use super::types::*;

#[repr(u8)]
pub enum Castling {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
    None,
}

pub struct MoveFlags;

#[allow(non_upper_case_globals)]
impl MoveFlags {
    pub const K: u8 = 1u8 << Castling::WhiteKingSide as u8;
    pub const Q: u8 = 1u8 << Castling::WhiteQueenSide as u8;
    pub const k: u8 = 1u8 << Castling::BlackKingSide as u8;
    pub const q: u8 = 1u8 << Castling::BlackQueenSide as u8;
    pub const KQ: u8 = Self::K | Self::Q;
    pub const kq: u8 = Self::k | Self::q;
    pub const KQkq: u8 = Self::KQ | Self::kq;
}

pub struct Move {
    pub from_sq: u8,
    pub to_sq: u8,
    pub pieces: u8, // encode from piece and to piece,
    pub flags: u8,  // reserved for castling, en passant, promotion
}

impl Move {
    const PIECE_MASK: u8 = 0xF;
    const CAPTURE_MASK: u8 = 0xF0;

    pub fn new(from_sq: u8, to_sq: u8, piece: Piece, capture: Piece, flags: u8) -> Self {
        debug_assert!(from_sq < 64 && to_sq < 64);
        debug_assert!(piece != Piece::None);

        let pieces = (piece as u8) & Self::PIECE_MASK | ((capture as u8) << 4) & Self::CAPTURE_MASK;
        Self { from_sq, to_sq, pieces, flags }
    }

    pub fn piece(&self) -> Piece {
        let bits = self.pieces & 0b1111;
        let piece = unsafe { std::mem::transmute(bits) };
        piece
    }

    pub fn capture(&self) -> Piece {
        let bits = (self.pieces & Self::CAPTURE_MASK) >> 4;
        unsafe { std::mem::transmute(bits) }
    }

    pub fn castling_mask(&self) -> u8 {
        self.flags & MoveFlags::KQkq
    }

    pub fn castling_type(&self) -> Castling {
        match self.piece() {
            Piece::WKing if self.to_sq == SQ_G1 => Castling::WhiteKingSide,
            Piece::WKing if self.to_sq == SQ_C1 => Castling::WhiteQueenSide,
            Piece::BKing if self.to_sq == SQ_G8 => Castling::BlackKingSide,
            Piece::BKing if self.to_sq == SQ_C8 => Castling::BlackQueenSide,
            _ => Castling::None,
        }
    }
}

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

// if castling rights are already disabled, return
// if king moved, disable castling rights, return
// if rook moved, disable castling rights, return
// if rook taken out, disable castling rights, return
fn move_disable_castling<const BIT: u8>(pos: &Position, from: Piece, to: Piece, from_sq: u8, to_sq: u8) -> u8 {
    if pos.state.castling & BIT == 0 {
        return 0;
    }

    if from == Piece::WKing && (BIT & MoveFlags::KQ) != 0 {
        return BIT;
    }

    if from == Piece::BKing && (BIT & MoveFlags::kq) != 0 {
        return BIT;
    }

    match (from, from_sq) {
        (Piece::WRook, SQ_A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
        (Piece::WRook, SQ_H1) if (BIT & MoveFlags::K) != 0 => return BIT,
        (Piece::BRook, SQ_A8) if (BIT & MoveFlags::q) != 0 => return BIT,
        (Piece::BRook, SQ_H8) if (BIT & MoveFlags::k) != 0 => return BIT,
        _ => {}
    }

    match (to, to_sq) {
        (Piece::WRook, SQ_A1) if (BIT & MoveFlags::Q) != 0 => return BIT,
        (Piece::WRook, SQ_H1) if (BIT & MoveFlags::K) != 0 => return BIT,
        (Piece::BRook, SQ_A8) if (BIT & MoveFlags::q) != 0 => return BIT,
        (Piece::BRook, SQ_H8) if (BIT & MoveFlags::k) != 0 => return BIT,
        _ => {}
    }

    0
}

// @TODO: restore castling rights if the move is undone

pub fn create_move(pos: &Position, from_sq: u8, to_sq: u8) -> Option<Move> {
    if !pos.occupancies[pos.state.side_to_move as usize].has_bit(from_sq) {
        return None;
    }

    let mut from = Piece::None;
    let mut to = Piece::None;
    for i in 0..pos.state.bitboards.len() {
        let bb = &pos.state.bitboards[i];
        if bb.has_bit(from_sq) {
            from = unsafe { std::mem::transmute(i as u8) };
        }
        if bb.has_bit(to_sq) {
            to = unsafe { std::mem::transmute(i as u8) };
        }
    }

    debug_assert!(from != Piece::None, "No piece found on 'from' square");
    let mut flags = 0u8;
    flags |= move_disable_castling::<{ MoveFlags::K }>(pos, from, to, from_sq, to_sq);
    flags |= move_disable_castling::<{ MoveFlags::Q }>(pos, from, to, from_sq, to_sq);
    flags |= move_disable_castling::<{ MoveFlags::k }>(pos, from, to, from_sq, to_sq);
    flags |= move_disable_castling::<{ MoveFlags::q }>(pos, from, to, from_sq, to_sq);

    Some(Move::new(from_sq, to_sq, from, to, flags))
}

fn do_move_generic(pos: &mut Position, m: &Move) {
    debug_assert!(pos.occupancies[pos.state.side_to_move as usize].has_bit(m.from_sq));

    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.state.bitboards[from as usize];

    bb_from.unset_bit(m.from_sq); // Clear the 'from' square
    bb_from.set_bit(m.to_sq); // Place piece on 'to' square
    if to != Piece::None {
        pos.state.bitboards[to as usize].unset_bit(m.to_sq); // Clear the 'to' square for the captured piece
    }
}

fn undo_move_generic(pos: &mut Position, m: &Move) {
    let from = m.piece();
    let to = m.capture();

    let bb_from = &mut pos.state.bitboards[from as usize];
    bb_from.set_bit(m.from_sq); // Place piece back on 'from' square
    bb_from.unset_bit(m.to_sq); // Clear the 'to' square

    if to != Piece::None {
        pos.state.bitboards[to as usize].set_bit(m.to_sq); // Place captured piece back on 'to' square
    }
}

fn do_castling(pos: &mut Position, m: &Move) {
    // Update castling rights if necessary
    pos.state.castling &= !m.castling_mask();

    // Move rook to its new position
    match m.castling_type() {
        Castling::WhiteKingSide => {
            pos.state.bitboards[Piece::WRook as usize].unset_bit(SQ_H1);
            pos.state.bitboards[Piece::WRook as usize].set_bit(SQ_F1);
        }
        Castling::WhiteQueenSide => {
            pos.state.bitboards[Piece::WRook as usize].unset_bit(SQ_A1);
            pos.state.bitboards[Piece::WRook as usize].set_bit(SQ_D1);
        }
        Castling::BlackKingSide => {
            pos.state.bitboards[Piece::BRook as usize].unset_bit(SQ_H8);
            pos.state.bitboards[Piece::BRook as usize].set_bit(SQ_F8);
        }
        Castling::BlackQueenSide => {
            pos.state.bitboards[Piece::BRook as usize].unset_bit(SQ_A8);
            pos.state.bitboards[Piece::BRook as usize].set_bit(SQ_D8);
        }
        Castling::None => {}
    }
}

fn undo_castling(pos: &mut Position, m: &Move) {
    // Restore castling rights if necessary
    pos.state.castling |= m.castling_mask();

    // Move rook back to its original position
    match m.castling_type() {
        Castling::WhiteKingSide => {
            pos.state.bitboards[Piece::WRook as usize].unset_bit(SQ_F1);
            pos.state.bitboards[Piece::WRook as usize].set_bit(SQ_H1);
        }
        Castling::WhiteQueenSide => {
            pos.state.bitboards[Piece::WRook as usize].unset_bit(SQ_D1);
            pos.state.bitboards[Piece::WRook as usize].set_bit(SQ_A1);
        }
        Castling::BlackKingSide => {
            pos.state.bitboards[Piece::BRook as usize].unset_bit(SQ_F8);
            pos.state.bitboards[Piece::BRook as usize].set_bit(SQ_H8);
        }
        Castling::BlackQueenSide => {
            pos.state.bitboards[Piece::BRook as usize].unset_bit(SQ_D8);
            pos.state.bitboards[Piece::BRook as usize].set_bit(SQ_A8);
        }
        Castling::None => {}
    }
}

fn post_move(pos: &mut Position) {
    pos.state.change_side();
    pos.update_cache();
}

pub fn do_move(pos: &mut Position, m: &Move) -> bool {
    do_move_generic(pos, m);
    do_castling(pos, m);
    post_move(pos);
    true
}

pub fn undo_move(pos: &mut Position, m: &Move) {
    undo_move_generic(pos, m);
    undo_castling(pos, m);
    post_move(pos);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let m = Move::new(SQ_E7, SQ_E8, Piece::WQueen, Piece::BKnight, 0);
        assert_eq!(m.piece(), Piece::WQueen);
        assert_eq!(m.capture(), Piece::BKnight);

        let m = Move::new(SQ_E7, SQ_E8, Piece::BQueen, Piece::None, 0);
        assert_eq!(m.piece(), Piece::BQueen);
        assert_eq!(m.capture(), Piece::None);
    }

    #[test]
    fn test_parse_move() {
        assert_eq!(parse_move("e2e4"), Some((SQ_E2, SQ_E4)));
        assert_eq!(parse_move("a7a8"), Some((SQ_A7, SQ_A8)));
        assert_eq!(parse_move("h1h2"), Some((SQ_H1, SQ_H2)));
        assert_eq!(parse_move("d4d5"), Some((SQ_D4, SQ_D5)));
        assert_eq!(parse_move("z1z2"), None);
        assert_eq!(parse_move("e9e4"), None);
        assert_eq!(parse_move("e2e"), None);
    }
}
