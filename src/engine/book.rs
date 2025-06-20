use std::collections::HashMap;

use crate::core::types::{File, Move, MoveType, PieceType, Rank, Square};
use crate::core::zobrist::Zobrist;
use crate::logger;

static BOOK_DATA: &[u8] = include_bytes!("./gm2600.bin");

pub struct BookEntry {
    raw_move: u16, // 2 bytes
    weight: u16,   // 2 bytes
}

impl BookEntry {
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

    pub fn weight(&self) -> u16 {
        self.weight
    }

    pub fn get_promotion(&self) -> Option<PieceType> {
        let val = (self.raw_move >> 12) & 0x0007;
        match val {
            0 => None,
            1..=4 => Some(PieceType(val as u8)),
            _ => panic!("Invalid promotion piece value: {}", val),
        }
    }

    pub fn to_move(&self) -> Move {
        const WHITE_KING_SIDE: u16 = 0x0107; // e1g1
        const WHITE_QUEEN_SIDE: u16 = 0x0f3f; // e1c1
        const BLACK_KING_SIDE: u16 = 0x0100; // e8g8
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
pub struct Book {
    pub map: HashMap<Zobrist, Vec<BookEntry>>,
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
            self.map.entry(Zobrist(key)).or_insert(Vec::new()).push(entry);
        }

        logger::log(format!("Loaded {} book entries", self.map.len()));

        Ok(())
    }

    pub fn get_move(&self, hash: Zobrist) -> Option<Move> {
        if let Some(entries) = self.map.get(&hash) {
            // @TODO: sort entries by weight and return the best one
            assert!(!entries.is_empty(), "No entries found for hash: {:?}", hash);
            return Some(entries[0].to_move());
        }

        None
    }
}

use once_cell::sync::Lazy;

// Global, accessible everywhere
pub static DEFAULT_BOOK: Lazy<Book> = Lazy::new(|| Book::default());

#[cfg(test)]
mod tests {}
