use super::fen_state::FenState;
use super::{fen_state, types::*};

pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flags: u8, // encode captures, promotion and en passant
}

const PROMO_MASK: u8 = 0b111;
const CAPTURE_MASK: u8 = 0b111000;

impl Move {
    pub fn new(from: u8, to: u8, promo: PieceType, capture: PieceType) -> Self {
        assert!(from < 64 && to < 64, "Invalid square index");
        assert!(promo != PieceType::Pawn && promo != PieceType::King, "Invalid promotion piece type");

        let mut flags = (promo as u8) & PROMO_MASK;
        flags |= (capture as u8) << 3;
        Self { from, to, flags }
    }

    pub fn promotion(&self) -> PieceType {
        let promo = unsafe { std::mem::transmute(self.flags & 0b111) };
        assert!(promo != PieceType::Pawn && promo != PieceType::King, "Invalid promotion piece type");
        promo
    }

    pub fn capture(&self) -> PieceType {
        let bits = (self.flags & CAPTURE_MASK) >> 3;
        unsafe { std::mem::transmute(bits) }
    }
}

pub struct Position {
    pub state: FenState,
    pub occupancies: [u64; 3],
    history: Vec<Move>,
}

impl Position {
    pub fn new() -> Self {
        let state = FenState::new();
        let occupancies = fen_state::occupancies(&state);
        Self { state, occupancies, history: Vec::new() }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let state = FenState::from_fen(fen)?;
        let occupancies = fen_state::occupancies(&state);
        Ok(Self { state, occupancies, history: Vec::new() })
    }

    pub fn do_move(&mut self, m: Move) -> bool {
        false
    }

    pub fn undo_move(&mut self, m: Move) -> bool {
        false
    }

    pub fn apply_move(&mut self, from_sq: u8, to_sq: u8) -> bool {
        let from_mask = 1u64 << from_sq;
        let to_mask = 1u64 << to_sq;
        if self.occupancies[self.state.side_to_move as usize] & from_mask == 0 {
            return false;
        }

        let bitboards = fen_state::to_mut_vec(&mut self.state);

        let mut from = Piece::None;
        let mut to = Piece::None;
        for i in 0..bitboards.len() {
            if (*bitboards[i] & from_mask) != 0 {
                from = unsafe { std::mem::transmute(i as u8) };
            }
            if (*bitboards[i] & to_mask) != 0 {
                to = unsafe { std::mem::transmute(i as u8) };
            }
        }

        assert!(from != Piece::None, "No piece found on 'from' square");

        *bitboards[from as usize] &= !from_mask; // Remove piece from 'from' square
        *bitboards[from as usize] |= to_mask; // Place piece on 'to' square
        if to != Piece::None {
            *bitboards[to as usize] &= !to_mask; // Clear the 'to' square for the captured piece
        }

        self.history.push(Move::new(from_sq, to_sq, PieceType::None, PieceType::None));

        // @TODO: Handle captures, promotions, en passant, castling
        self.state.side_to_move = get_opposite_color(self.state.side_to_move);

        self.occupancies = fen_state::occupancies(&self.state);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let m = Move::new(SQ_E7, SQ_E8, PieceType::Queen, PieceType::Knight);
        assert_eq!(m.promotion(), PieceType::Queen);
        assert_eq!(m.capture(), PieceType::Knight);

        let m = Move::new(SQ_E7, SQ_E8, PieceType::None, PieceType::None);
        assert_eq!(m.promotion(), PieceType::None);
        assert_eq!(m.capture(), PieceType::None);
    }
}
