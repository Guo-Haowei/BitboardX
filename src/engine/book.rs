use std::collections::HashMap;

use crate::core::types::{CastlingType, File, PieceType, Rank, Square};

static BOOK_DATA: &[u8] = include_bytes!("./gm2600.bin");

pub struct BookEntry {
    key: u64,      // 8 bytes
    raw_move: u16, // 2 bytes
    weight: u16,   // 2 bytes
}

impl BookEntry {
    pub fn new(key: u64, raw_move: u16, weight: u16) -> Self {
        Self { key, raw_move, weight }
    }

    pub fn key(&self) -> u64 {
        self.key
    }

    pub fn src_sq(&self) -> Square {
        let rank = (self.raw_move >> 9) & 0x0007; // bits 9-11
        let file = (self.raw_move >> 6) & 0x0007; // bits 6-8

        Square::make(File(file as u8), Rank(rank as u8))
    }

    pub fn dst_sq(&self) -> Square {
        let rank = (self.raw_move >> 3) & 0x0007; // bits 3-5
        let file = self.raw_move & 0x0007; // bits 0-2

        Square::make(File(file as u8), Rank(rank as u8))
    }

    pub fn castling(&self) -> CastlingType {
        match self.raw_move {
            0x0107 => CastlingType::WhiteKingSide,  // e1g1
            0x0f3f => CastlingType::WhiteQueenSide, // e1c1
            0x0100 => CastlingType::BlackKingSide,  // e8g8
            0x0f38 => CastlingType::BlackQueenSide, // e8c8
            _ => CastlingType::None,
        }
    }

    pub fn weight(&self) -> u16 {
        self.weight
    }

    pub fn get_promo_piece(&self) -> PieceType {
        let val = (self.raw_move >> 12) & 0x0007;
        match val {
            0 => PieceType::NONE,
            1..=4 => PieceType(val as u8),
            _ => panic!("Invalid promotion piece value: {}", val),
        }
    }
}

// Maybe we can use fixed size array for list of moves
pub struct Book {
    pub map: HashMap<u64, Vec<BookEntry>>,
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
            let entry = BookEntry { key, raw_move, weight };
            self.map.entry(key).or_insert(Vec::new()).push(entry);
        }

        println!("Loaded {} book entries", self.map.len());

        Ok(())
    }
}

use once_cell::sync::Lazy;

// Global, accessible everywhere
pub static DEFAULT_BOOK: Lazy<Book> = Lazy::new(|| Book::default());

#[cfg(test)]
mod tests {}
