use super::player::{Player, PlayerAction};
use crate::core::{move_gen, position::Position};
use std::any::Any;

pub struct AiPlayer {
    rand: DummyRng,
}

impl Player for AiPlayer {
    fn name(&self) -> String {
        "AiPlayer".to_string()
    }

    fn request_move(&mut self) {}

    fn poll_move(&mut self, fen: String) -> PlayerAction {
        match Position::from_fen(fen.as_str()) {
            Ok(pos) => {
                let moves = move_gen::legal_moves(&pos);

                let n = self.rand.next_u32(&fen);
                let n = n % moves.len() as u32;
                let m = moves.get(n as usize).unwrap().to_string();

                PlayerAction::Ready(m)
            }
            Err(_) => PlayerAction::Error("Invalid FEN string".to_string()),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AiPlayer {
    pub fn new() -> Self {
        Self {
            rand: DummyRng::new(0x12345678), // Example seed
        }
    }
}

pub struct DummyRng {
    state: u64,
}

impl DummyRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Mix the string into the internal state
    fn mix_in_str(&mut self, input: &str) {
        for b in input.bytes() {
            self.state ^= b as u64;
            self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        }
    }

    /// Call this with a string each time you want a new number
    pub fn next_u32(&mut self, extra: &str) -> u32 {
        self.mix_in_str(extra);
        // LCG step
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 16) as u32
    }

    pub fn next_f64(&mut self, extra: &str) -> f64 {
        self.next_u32(extra) as f64 / (u32::MAX as f64 + 1.0)
    }

    pub fn gen_range(&mut self, min: u32, max: u32, extra: &str) -> u32 {
        min + self.next_u32(extra) % (max - min)
    }
}
