use crate::{board, fen_state, moves::*, types::*};
use fen_state::FenState;

pub struct Board {
    pub state: FenState,
    pub occupancies: [u64; 3],
}

impl Board {
    pub fn new() -> Self {
        let state = FenState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let occupancies = fen_state::occupancies(&state);
        Self { state, occupancies }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let state = FenState::from_fen(fen)?;
        let occupancies = fen_state::occupancies(&state);
        Ok(Self { state, occupancies })
    }

    pub fn apply_move(&mut self, from: u8, to: u8) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;
        if self.occupancies[self.state.side_to_move as usize] & from_mask == 0 {
            return false;
        }

        let moves = board::gen_moves(self, from);
        if moves & to_mask == 0 {
            return false;
        }

        let mut index: i8 = -1;
        let mut bitboards = fen_state::to_mut_vec(&mut self.state);
        // let mut index: usize = self.bitboards.len();
        for i in 0..bitboards.len() {
            if (*bitboards[i] & from_mask) != 0 {
                index = i as i8;
            }
            *bitboards[i] &= !to_mask; // Clear the 'to' square for all pieces
        }

        if index != -1 {
            *bitboards[index as usize] &= !from_mask; // Remove piece from 'from' square
            *bitboards[index as usize] |= to_mask; // Place piece on 'to' square
        }

        self.occupancies = fen_state::occupancies(&self.state);

        self.state.side_to_move = get_opposite_color(self.state.side_to_move);

        true
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
