use super::fen_state::FenState;
use super::{fen_state, types::*};

pub struct Move {
    pub from_sq: u8,
    pub to_sq: u8,
    pub pieces: u8, // encode from piece and to piece,
                    // @TODO: promotion, en passant, castling
}

pub struct Position {
    pub state: FenState,
    pub occupancies: [u64; 3],
}

impl Move {
    const PIECE_MASK: u8 = 0xF;
    const CAPTURE_MASK: u8 = 0xF0;

    pub fn new(from_sq: u8, to_sq: u8, piece: Piece, capture: Piece) -> Self {
        assert!(from_sq < 64 && to_sq < 64);
        assert!(piece != Piece::None);

        let pieces = (piece as u8) & Self::PIECE_MASK | ((capture as u8) << 4) & Self::CAPTURE_MASK;
        Self { from_sq, to_sq, pieces }
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

impl Position {
    pub fn new() -> Self {
        let state = FenState::new();
        let occupancies = fen_state::occupancies(&state);
        Self { state, occupancies }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let state = FenState::from_fen(fen)?;
        let occupancies = fen_state::occupancies(&state);
        Ok(Self { state, occupancies })
    }

    pub fn do_move(&mut self, m: &Move) -> bool {
        let from_mask = 1u64 << m.from_sq;
        let to_mask = 1u64 << m.to_sq;
        if self.occupancies[self.state.side_to_move as usize] & from_mask == 0 {
            return false;
        }

        let from = m.piece();
        let to = m.capture();

        let bb_attack = &mut self.state.bitboards[from as usize];

        *bb_attack &= !from_mask; // Remove piece from 'from' square
        *bb_attack |= to_mask; // Place piece on 'to' square
        if to != Piece::None {
            self.state.bitboards[to as usize] &= !to_mask; // Clear the 'to' square for the captured piece
        }

        // @TODO: extract to a function
        self.state.side_to_move = get_opposite_color(self.state.side_to_move);
        self.occupancies = fen_state::occupancies(&self.state);

        true
    }

    pub fn undo_move(&mut self, m: &Move) {
        let from = m.piece();
        let to = m.capture();
        let from_mask = 1u64 << m.from_sq;
        let to_mask = 1u64 << m.to_sq;

        let bb_attack = &mut self.state.bitboards[from as usize];
        *bb_attack |= from_mask; // Place piece back on 'from' square
        *bb_attack &= !to_mask; // Remove piece from 'to' square

        self.state.bitboards[to as usize] |= to_mask; // Place piece back on 'to' square

        // @TODO: extract to a function
        self.state.side_to_move = get_opposite_color(self.state.side_to_move);
        self.occupancies = fen_state::occupancies(&self.state);

        panic!("not implemented yet");
    }

    pub fn create_move(&self, from_sq: u8, to_sq: u8) -> Option<Move> {
        let from_mask = 1u64 << from_sq;
        let to_mask = 1u64 << to_sq;
        if self.occupancies[self.state.side_to_move as usize] & from_mask == 0 {
            return None;
        }

        let mut from = Piece::None;
        let mut to = Piece::None;
        for i in 0..self.state.bitboards.len() {
            let bb = self.state.bitboards[i];
            if (bb & from_mask) != 0 {
                from = unsafe { std::mem::transmute(i as u8) };
            }
            if (bb & to_mask) != 0 {
                to = unsafe { std::mem::transmute(i as u8) };
            }
        }

        assert!(from != Piece::None, "No piece found on 'from' square");

        Some(Move::new(from_sq, to_sq, from, to))
    }
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
}
