use super::position::Position;
use super::types::*;

// castling rights
pub struct MoveFlags;

#[allow(non_upper_case_globals)]
impl MoveFlags {
    pub const K: u8 = 0b0001;
    pub const Q: u8 = 0b0010;
    pub const k: u8 = 0b0100;
    pub const q: u8 = 0b1000;
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

    pub fn new(from_sq: u8, to_sq: u8, piece: Piece, capture: Piece) -> Self {
        assert!(from_sq < 64 && to_sq < 64);
        assert!(piece != Piece::None);

        let pieces = (piece as u8) & Self::PIECE_MASK | ((capture as u8) << 4) & Self::CAPTURE_MASK;
        let flags = 0;
        Self { from_sq, to_sq, pieces, flags }
    }

    pub fn piece(&self) -> Piece {
        let flag = unsafe { std::mem::transmute(self.pieces & 0b1111) };
        flag
    }

    pub fn capture(&self) -> Piece {
        let flag = (self.pieces & Self::CAPTURE_MASK) >> 4;
        unsafe { std::mem::transmute(flag) }
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

    assert!(from != Piece::None, "No piece found on 'from' square");

    // if from == Piece::WKing && pos.state.castling & Castling::WK.bits() != 0 {}

    // // check if castling
    // if from_sq == SQ_E1 && to_sq == SQ_G1 { // White kingside castling
    //     //
    // }

    Some(Move::new(from_sq, to_sq, from, to))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let m = Move::new(SQ_E7, SQ_E8, Piece::WQueen, Piece::BKnight);
        assert_eq!(m.piece(), Piece::WQueen);
        assert_eq!(m.capture(), Piece::BKnight);

        let m = Move::new(SQ_E7, SQ_E8, Piece::BQueen, Piece::None);
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
