use crate::engine::move_gen;

use super::bitboard::BitBoard;
use super::fen_state::FenState;
use super::{fen_state, types::*};

pub struct Position {
    pub state: FenState,
    pub occupancies: [BitBoard; 3],
    pub attack_map: [BitBoard; NB_COLORS],
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

    pub fn update_cache(&mut self) {
        self.occupancies = fen_state::occupancies(&self.state);

        // maybe only need to update the attack map for the inactive side
        self.attack_map[Color::White as usize] = self.attack_map::<true, W_START, W_END>();
        self.attack_map[Color::Black as usize] = self.attack_map::<false, B_START, B_END>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_map() {
        let pos = Position::new();
        let white_attack_map = pos.attack_map::<true, W_START, W_END>();
        let black_attack_map = pos.attack_map::<false, B_START, B_END>();

        assert_eq!(white_attack_map.get(), 0x0000000000FF0000);
        assert_eq!(black_attack_map.get(), 0x0000FF0000000000);
    }
}
