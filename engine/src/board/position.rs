use crate::engine::move_gen;

use super::bitboard::BitBoard;
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
    pub occupancies: [BitBoard; 3],
    pub attack_map: [BitBoard; NB_COLORS],
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
        let mut pos = Self { state, occupancies: [BitBoard::new(); 3], attack_map: [BitBoard::new(); NB_COLORS] };

        pos.update_cache();
        pos
    }

    pub fn from(
        piece_placement: &str,
        side_to_move: &str,
        castling_rights: &str,
        en_passant_target: &str,
        halfmove_clock: &str,
        fullmove_number: &str,
    ) -> Result<Self, String> {
        let state = FenState::from(
            piece_placement,
            side_to_move,
            castling_rights,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
        )?;
        let mut pos = Self { state, occupancies: [BitBoard::new(); 3], attack_map: [BitBoard::new(); NB_COLORS] };

        pos.update_cache();
        Ok(pos)
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields".to_string());
        }

        Self::from(parts[0], parts[1], parts[2], parts[3], parts[4], parts[5])
    }

    fn attack_map<const IS_WHITE: bool, const START: u8, const END: u8>(&self) -> BitBoard {
        let mut attack_map = BitBoard::new();
        let color = if IS_WHITE { Color::White } else { Color::Black };

        for i in START..END {
            // pieces from W to B
            let bb = self.state.bitboards[i as usize].get();
            for f in 0..8 {
                for r in 0..8 {
                    let sq = make_square(f, r);
                    if bb & (1u64 << sq) != 0 {
                        attack_map |= move_gen::gen_attack_moves(self, sq, color);
                    }
                }
            }
        }

        attack_map
    }

    fn update_cache(&mut self) {
        self.occupancies = fen_state::occupancies(&self.state);

        // maybe only need to update the attack map for the inactive side
        self.attack_map[Color::White as usize] = self.attack_map::<true, W_START, W_END>();
        self.attack_map[Color::Black as usize] = self.attack_map::<false, B_START, B_END>();
    }

    pub fn do_move(&mut self, m: &Move) -> bool {
        if !self.occupancies[self.state.side_to_move as usize].has_bit(m.from_sq) {
            panic!("Invalid move: 'from' square does not contain a piece of the current side");
        }

        let from = m.piece();
        let to = m.capture();

        // @TODO: check to piece is rook, if so, disable castling rights

        let bb_attack = &mut self.state.bitboards[from as usize];

        bb_attack.unset_bit(m.from_sq); // Clear the 'from' square
        bb_attack.set_bit(m.to_sq); // Place piece on 'to' square
        if to != Piece::None {
            self.state.bitboards[to as usize].unset_bit(m.to_sq); // Clear the 'to' square for the captured piece
        }

        self.state.change_side();
        self.update_cache();

        true
    }

    pub fn undo_move(&mut self, m: &Move) {
        let from = m.piece();
        let to = m.capture();

        let bb_attack = &mut self.state.bitboards[from as usize];
        bb_attack.set_bit(m.from_sq); // Place piece back on 'from' square
        bb_attack.unset_bit(m.to_sq); // Clear the 'to' square

        if to != Piece::None {
            self.state.bitboards[to as usize].set_bit(m.to_sq); // Place captured piece back on 'to' square
        }

        self.state.change_side();
        self.update_cache();
    }

    pub fn create_move(&self, from_sq: u8, to_sq: u8) -> Option<Move> {
        if !self.occupancies[self.state.side_to_move as usize].has_bit(from_sq) {
            return None;
        }

        let mut from = Piece::None;
        let mut to = Piece::None;
        for i in 0..self.state.bitboards.len() {
            let bb = &self.state.bitboards[i];
            if bb.has_bit(from_sq) {
                from = unsafe { std::mem::transmute(i as u8) };
            }
            if bb.has_bit(to_sq) {
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

    #[test]
    fn test_attack_map() {
        let pos = Position::new();
        let white_attack_map = pos.attack_map::<true, W_START, W_END>();
        let black_attack_map = pos.attack_map::<false, B_START, B_END>();

        assert_eq!(white_attack_map.get(), 0x0000000000FF0000);
        assert_eq!(black_attack_map.get(), 0x0000FF0000000000);
    }
}
