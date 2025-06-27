use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::core::types::{File, Move, MoveType, PieceType, Rank, Square};
use crate::core::zobrist::ZobristHash;
use crate::utils::*;

static BOOK_DATA: &[u8] = include_bytes!("./gm2600.bin");

struct BookEntry {
    raw_move: u16, // 2 bytes
    weight: u16,   // 2 bytes
}

impl BookEntry {
    fn src_sq(&self) -> Square {
        let rank = (self.raw_move >> 9) & 0x0007; // bits 9-11
        let file = (self.raw_move >> 6) & 0x0007; // bits 6-8

        Square::make(File(file as u8), Rank(rank as u8))
    }

    fn dst_sq(&self) -> Square {
        let rank = (self.raw_move >> 3) & 0x0007; // bits 3-5
        let file = self.raw_move & 0x0007; // bits 0-2

        Square::make(File(file as u8), Rank(rank as u8))
    }

    fn get_promotion(&self) -> Option<PieceType> {
        let val = (self.raw_move >> 12) & 0x0007;
        match val {
            0 => None,
            1..=4 => Some(PieceType(val as u8)),
            _ => panic!("Invalid promotion piece value: {}", val),
        }
    }

    fn to_move(&self) -> Move {
        const WHITE_KING_SIDE: u16 = 0x0107; // e1g1
        const WHITE_QUEEN_SIDE: u16 = 0x0100; // e1c1
        const BLACK_KING_SIDE: u16 = 0x0f3f; // e8g8
        const BLACK_QUEEN_SIDE: u16 = 0x0f38; // e8c8

        match self.raw_move {
            WHITE_KING_SIDE => Move::new(Square::E1, Square::G1, MoveType::Castling, None),
            WHITE_QUEEN_SIDE => Move::new(Square::E1, Square::C1, MoveType::Castling, None),
            BLACK_KING_SIDE => Move::new(Square::E8, Square::G8, MoveType::Castling, None),
            BLACK_QUEEN_SIDE => Move::new(Square::E8, Square::C8, MoveType::Castling, None),
            _ => {
                let promotion = self.get_promotion();
                let src = self.src_sq();
                let dst = self.dst_sq();
                // this is a little bit hacky here,
                // because it's hard to tell if it's a en passant move or not
                Move::new(
                    src,
                    dst,
                    if promotion.is_none() { MoveType::Normal } else { MoveType::Promotion },
                    promotion,
                )
            }
        }
    }
}

// Maybe we can use fixed size array for list of moves
struct MoveList {
    moves: Vec<BookEntry>,
    total_weight: u32,
}

impl MoveList {
    fn new() -> Self {
        Self { moves: Vec::new(), total_weight: 0 }
    }

    fn post_load(&mut self) {
        // Sort moves by weight in descending order
        self.moves.sort_by_key(|entry| -i32::from(entry.weight));
        // Calculate total weight for weighted random selection
        self.total_weight = self.moves.iter().map(|entry| entry.weight as u32).sum();
    }
}

pub struct Book {
    map: HashMap<ZobristHash, MoveList>,
}

impl Book {
    pub fn default() -> Self {
        let mut book = Self { map: HashMap::new() };
        book.load(BOOK_DATA).unwrap();
        book
    }

    pub fn load(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Book data is empty".to_string());
        }

        let mut offset = 0;

        while offset < data.len() {
            if data.len() - offset < 16 {
                return Err("Incomplete book entry".to_string());
            }

            let key = u64::from_be_bytes(data[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let raw_move = u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap());
            offset += 2;

            let weight = u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap());
            offset += 2;

            // don't care about learn for now
            let _learn = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
            offset += 4;

            // Create a BookEntry
            let entry = BookEntry { raw_move, weight };
            self.map.entry(ZobristHash(key)).or_insert(MoveList::new()).moves.push(entry);
        }

        for move_list in self.map.values_mut() {
            move_list.post_load();
        }

        Ok(())
    }

    pub fn get_move(&self, hash: ZobristHash) -> Option<Move> {
        if let Some(entries) = self.map.get(&hash) {
            debug_assert!(!entries.moves.is_empty(), "No entries found for hash: {:?}", hash);
            let rand = random();
            let mut random_weight = (rand * entries.total_weight as f32) as i16;
            random_weight = random_weight.min(entries.total_weight as i16); // Ensure non-negative
            let mut entry: Option<&BookEntry> = None;
            for e in &entries.moves {
                random_weight -= e.weight as i16;
                if random_weight <= 0 {
                    entry = Some(e);
                    break;
                }
            }
            debug_assert!(entry.is_some());

            let entry = entry.unwrap();
            let mv = entry.to_move();
            log::debug!(
                "found book move: {} out of {} moves, rand: {} (weight: {}/{})",
                mv.to_string(),
                entries.moves.len(),
                rand,
                entry.weight,
                entries.total_weight
            );
            return Some(mv);
        }

        None
    }
}

pub static DEFAULT_BOOK: Lazy<Book> = Lazy::new(|| Book::default());

#[cfg(test)]
mod tests {}
