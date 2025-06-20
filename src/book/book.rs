use crate::core::types::{CastlingType, File, Piece, PieceType, Rank, Square};

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

pub struct Book {}

pub fn load(data: &[u8]) -> Result<(), String> {
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

        let castling = entry.castling();
        match castling {
            CastlingType::WhiteKingSide => {
                println!(
                    "White King-side castling move detected: from {:?} to {:?}",
                    entry.src_sq().to_string(),
                    entry.dst_sq().to_string()
                );
            }
            CastlingType::BlackKingSide => {
                println!(
                    "Black King-side castling move detected: from {:?} to {:?}",
                    entry.src_sq().to_string(),
                    entry.dst_sq().to_string()
                );
            }
            CastlingType::WhiteQueenSide => {
                println!(
                    "White Queen-side castling move detected: from {:?} to {:?}",
                    entry.src_sq().to_string(),
                    entry.dst_sq().to_string()
                );
            }
            CastlingType::BlackQueenSide => {
                println!(
                    "Black Queen-side castling move detected: from {:?} to {:?}",
                    entry.src_sq().to_string(),
                    entry.dst_sq().to_string()
                );
            }
            CastlingType::None => {}
        }

        let piece_type = entry.get_promo_piece();
        if piece_type != PieceType::NONE {
            println!(
                "Promotion move detected: from {:?} to {:?}, promote to {:?}",
                entry.src_sq().to_string(),
                entry.dst_sq().to_string(),
                piece_type
            );
        }

        println!(
            "** pos: {}, entry: from {:?} to {:?}, weight: {}, learn: {}",
            entry.key(),
            entry.src_sq().to_string(),
            entry.dst_sq().to_string(),
            entry.weight(),
            _learn
        );
    }

    // // Remember where we started from
    //     // noinspection JSUnusedGlobalSymbols
    //     this.ofs = ofs
    //     // Convert key to BigInt (64-bit) value
    //     let key = BigInt(0)
    //     let i
    //     let byt
    //     for (i = 0; i < 8; ++i) {
    //         byt = bookdata.charCodeAt(ofs++)
    //         key = (key << BigInt(8)) | BigInt(byt)
    //     }
    //     this.key = key
    //     let raw_move = 0
    //     for (i = 0; i < 2; ++i) {
    //         byt = bookdata.charCodeAt(ofs++)
    //         raw_move = (raw_move << 8) | byt
    //     }
    //     this.raw_move = raw_move
    //     let weight = 0
    //     for (i = 0; i < 2; ++i) {
    //         byt = bookdata.charCodeAt(ofs++)
    //         weight = (weight << 8) | byt
    //     }
    //     this.weight = weight
    //     let learn = 0
    //     for (i = 0; i < 4; ++i) {
    //         byt = bookdata.charCodeAt(ofs++)
    //         learn = (learn << 8) | byt
    //     }
    //     // noinspection JSUnusedGlobalSymbols
    //     this.learn = learn

    // Ok(data.to_vec())
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_loader() {
        load(BOOK_DATA).expect("Failed to load book data");
        // let book = BookLoader::new(BOOK_DATA);
        // assert!(!book.is_empty());

        // let pos =
        //     Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        // let moves = book.get_moves(&pos);
        // assert!(!moves.is_empty());
    }
}
