use once_cell::sync::Lazy;
use std::io::Cursor;
use std::io::{self, Read, Write};

use crate::core::types::{BitBoard, Square};

#[derive(Debug)]
pub struct MagicEntry {
    mask: u64,
    magic: u64,
    shift: u32,
    attack_table: Vec<u64>,
}

static MAGIC_BIN_ROOK: &[u8] = include_bytes!("magic/rook_magic.bin");
static MAGIC_BIN_BISHOP: &[u8] = include_bytes!("magic/bishop_magic.bin");

fn load_magic(bin: &[u8]) -> Vec<MagicEntry> {
    let mut entries = Vec::with_capacity(64);

    let mut cursor = Cursor::new(bin);
    while let Ok(entry) = MagicEntry::deserialize(&mut cursor) {
        entries.push(entry);
    }

    assert_eq!(entries.len(), 64, "Expected 64 magic entries, found {}", entries.len());
    entries
}

static MAGIC_TABLE_ROOK: Lazy<Vec<MagicEntry>> = Lazy::new(|| load_magic(MAGIC_BIN_ROOK));
static MAGIC_TABLE_BISHOP: Lazy<Vec<MagicEntry>> = Lazy::new(|| load_magic(MAGIC_BIN_BISHOP));

pub fn get_rook_attack_mask(blockers: BitBoard, square: Square) -> BitBoard {
    let magic_entry = &MAGIC_TABLE_ROOK[square.as_u8() as usize];
    let relevant_blockers = blockers & BitBoard::from(magic_entry.mask);

    let index = (relevant_blockers.get().wrapping_mul(magic_entry.magic)) >> magic_entry.shift;

    BitBoard::from(magic_entry.attack_table[index as usize])
}

pub fn get_bishop_attack_mask(blockers: BitBoard, square: Square) -> BitBoard {
    let magic_entry = &MAGIC_TABLE_BISHOP[square.as_u8() as usize];
    let relevant_blockers = blockers & BitBoard::from(magic_entry.mask);

    let index = (relevant_blockers.get().wrapping_mul(magic_entry.magic)) >> magic_entry.shift;

    BitBoard::from(magic_entry.attack_table[index as usize])
}

impl MagicEntry {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.mask.to_le_bytes())?;
        writer.write_all(&self.magic.to_le_bytes())?;
        writer.write_all(&self.shift.to_le_bytes())?;

        let len = self.attack_table.len() as u32;
        writer.write_all(&len.to_le_bytes())?;

        for &val in &self.attack_table {
            writer.write_all(&val.to_le_bytes())?;
        }
        Ok(())
    }

    // Deserialize from reader
    pub fn deserialize<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 8];

        reader.read_exact(&mut buf)?;
        let mask = u64::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let magic = u64::from_le_bytes(buf);

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let shift = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let len = u32::from_le_bytes(buf);

        let mut buf = [0u8; 8];
        let mut attack_table = Vec::with_capacity(len as usize);
        for _ in 0..len {
            reader.read_exact(&mut buf)?;
            attack_table.push(u64::from_le_bytes(buf));
        }

        Ok(Self { mask, magic, shift, attack_table })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::position::Position;
    use crate::core::types::{BitBoard, File, Rank, Square};

    use std::collections::HashSet;
    use std::io::BufWriter;

    /// Generate mask excluding edges on rank and file for rook
    /// Don't include the square itself,
    /// and don't include edges, because sliding pieces stops at edges anyway
    /// e.g. for square D4, the mask will look like this:
    /// a b c d e f g h
    /// 0 0 0 0 0 0 0 0 | 8
    /// 0 0 0 1 0 0 0 0 | 7
    /// 0 0 0 1 0 0 0 0 | 6
    /// 0 0 0 1 0 0 0 0 | 5
    /// 0 1 1 0 1 1 1 0 | 4
    /// 0 0 0 1 0 0 0 0 | 3
    /// 0 0 0 1 0 0 0 0 | 2
    /// 0 0 0 0 0 0 0 0 | 1
    const fn relevant_mask_rook(square: Square) -> BitBoard {
        let mut mask = BitBoard::new();

        let (file, rank) = square.file_rank();

        let mut f = File::B.0;
        while f <= File::G.0 {
            if f != file.0 {
                mask.set(Square::make(File(f), rank).as_u8());
            }
            f += 1;
        }

        let mut r = Rank::_2.0;
        while r <= Rank::_7.0 {
            if r != rank.0 {
                mask.set(Square::make(file, Rank(r)).as_u8());
            }
            r += 1;
        }

        mask
    }

    fn relevant_mask_bishop(square: Square) -> BitBoard {
        let mut mask = BitBoard::new();

        let (file, rank) = square.file_rank();

        let dirs = [
            (-1i8, -1i8), // up-left
            (1i8, 1i8),   // down-right
            (-1i8, 1i8),  // up-right
            (1i8, -1i8),  // down-left
        ];

        for (df, dr) in dirs {
            let mut f = file.0 as i8 + df;
            let mut r = rank.0 as i8 + dr;
            while f > 0 && f < 7 && r > 0 && r < 7 {
                let sq = Square::make(File(f as u8), Rank(r as u8));
                f += df;
                r += dr;
                mask.set(sq.as_u8());
            }
        }

        mask
    }

    fn find_blocker_combination_helper(
        mask: u64,
        so_far: u64,
        depth: u8,
        combinations: &mut Vec<u64>,
    ) {
        if depth == 0 {
            combinations.push(so_far);
            return;
        }

        let sq = mask.trailing_zeros() as u8;
        let sq_mask = 1u64 << sq;
        let mask = mask & (mask - 1);

        // either include the square or not
        find_blocker_combination_helper(mask, so_far, depth - 1, combinations);
        find_blocker_combination_helper(mask, so_far | sq_mask, depth - 1, combinations);
    }

    fn find_blocker_combination(mask: u64) -> Vec<u64> {
        let count = mask.count_ones() as u8;
        let combination_count = 1 << count; // 2^count combinations

        let mut combinations = Vec::with_capacity(combination_count);

        find_blocker_combination_helper(mask, 0, count, &mut combinations);

        debug_assert!(combinations.len() == combination_count);
        combinations
    }

    fn bake_attack_mask_rook(square: Square, blocker: u64) -> u64 {
        let blocker = BitBoard::from(blocker);
        let mut mask = BitBoard::new();
        let (file, rank) = square.file_rank();

        let dirs = [-1i8, 1i8];
        for dir in dirs {
            let mut f = file.0 as i8 + dir;
            while f >= 0 && f < 8 {
                let sq = Square::make(File(f as u8), rank);
                mask.set(sq.as_u8());
                if blocker.test(sq.as_u8()) {
                    break;
                }
                f += dir;
            }
        }

        for dir in dirs {
            let mut r = rank.0 as i8 + dir;
            while r >= 0 && r < 8 {
                let sq = Square::make(file, Rank(r as u8));
                mask.set(sq.as_u8());
                if blocker.test(sq.as_u8()) {
                    break;
                }
                r += dir;
            }
        }

        mask.get()
    }

    fn bake_attack_mask_bishop(square: Square, blocker: u64) -> u64 {
        let blocker = BitBoard::from(blocker);
        let mut mask = BitBoard::new();
        let (file, rank) = square.file_rank();

        let dirs = [(-1i8, -1i8), (1i8, 1i8), (-1i8, 1i8), (1i8, -1i8)];
        for (df, dr) in dirs {
            let mut f = file.0 as i8 + df;
            let mut r = rank.0 as i8 + dr;
            while f >= 0 && f < 8 && r >= 0 && r < 8 {
                let sq = Square::make(File(f as u8), Rank(r as u8));
                mask.set(sq.as_u8());
                if blocker.test(sq.as_u8()) {
                    break;
                }
                f += df;
                r += dr;
            }
        }

        mask.get()
    }

    fn find_magic(
        relevant_mask: u64,
        blockers: &[u64],
        attacks: &[u64],
    ) -> Option<(u64, u32, Vec<u64>)> {
        use rand::Rng;
        let relevant_bits = relevant_mask.count_ones() as u32;
        let shift = 64 - relevant_bits;

        let mut rng = rand::rng();

        for _ in 0..1_000_000 {
            let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();
            if magic.count_ones() < 6 {
                continue;
            }

            let size = 1 << relevant_bits;
            let mut table = vec![None; size];
            let mut success = true;

            for (&blocker, &attack) in blockers.iter().zip(attacks.iter()) {
                let index = ((blocker & relevant_mask).wrapping_mul(magic) >> shift) as usize;
                match table[index] {
                    None => table[index] = Some(attack),
                    Some(existing) if existing == attack => continue,
                    Some(_) => {
                        success = false;
                        break;
                    }
                }
            }

            if success {
                let attack_table = table.into_iter().map(|x| x.unwrap()).collect();
                return Some((magic, shift, attack_table));
            }
        }

        None
    }

    #[test]
    fn test_rook_relevant_blocker_mask() {
        let mask = relevant_mask_rook(Square::A1);

        assert_eq!(
            mask.to_string(),
            r#". . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
. X X X X X X .
"#
        );

        let mask = relevant_mask_rook(Square::D4);
        assert_eq!(
            mask.to_string(),
            r#". . . . . . . .
. . . X . . . .
. . . X . . . .
. . . X . . . .
. X X . X X X .
. . . X . . . .
. . . X . . . .
. . . . . . . .
"#
        );

        let mask = relevant_mask_bishop(Square::D4);
        assert_eq!(
            mask.to_string(),
            r#". . . . . . . .
. . . . . . X .
. X . . . X . .
. . X . X . . .
. . . . . . . .
. . X . X . . .
. X . . . X . .
. . . . . . . .
"#
        );
    }

    #[test]
    fn test_blocker_combination_generation() {
        let mask = relevant_mask_rook(Square::D4);
        let combinations = find_blocker_combination(mask.get());

        let set: HashSet<u64> = combinations.iter().copied().collect();
        assert_eq!(set.len(), combinations.len(), "Combinations should be unique");
    }

    #[test]
    fn test_bake_attacker_mask_rook() {
        /*
        . . . . . . . .
        . . . X . . . .
        . . . . . . . .
        . . . . . . . .
        . . . O . . . . // rook on D4
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        */
        let relevant_mask = relevant_mask_rook(Square::D4);
        let combinations = find_blocker_combination(relevant_mask.get());
        let blocker = combinations[1];
        let attack_mask = bake_attack_mask_rook(Square::D4, blocker);
        assert_eq!(
            BitBoard::from(attack_mask).to_string(),
            r#". . . . . . . .
. . . X . . . .
. . . X . . . .
. . . X . . . .
X X X . X X X X
. . . X . . . .
. . . X . . . .
. . . X . . . .
"#
        );
        // let blocker = BitBoard::from(0b
    }

    fn bake_magic_entry_rook(square: Square) -> MagicEntry {
        let mask = relevant_mask_rook(square).get();
        let blockers = find_blocker_combination(mask);
        let attacks: Vec<u64> =
            blockers.iter().map(|&b| bake_attack_mask_rook(square, b)).collect();

        let (magic, shift, table) = find_magic(mask, &blockers, &attacks).expect("No magic found");

        MagicEntry { magic, shift, mask, attack_table: table }
    }

    fn bake_magic_entry_bishop(square: Square) -> MagicEntry {
        let mask = relevant_mask_bishop(square).get();
        let blockers = find_blocker_combination(mask);
        let attacks: Vec<u64> =
            blockers.iter().map(|&b| bake_attack_mask_bishop(square, b)).collect();

        let (magic, shift, table) = find_magic(mask, &blockers, &attacks).expect("No magic found");

        MagicEntry { magic, shift, mask, attack_table: table }
    }

    #[test]
    fn test_magic_table_rook() {
        let fen = "8/2p5/3p4/KP5r/5R1k/8/4P1P1/8 w - - 0 1";
        let position = Position::from_fen(fen).unwrap();
        let square = Square::H5;
        let attack_mask = get_rook_attack_mask(position.state.occupancies[2], square);
        assert_eq!(
            attack_mask.to_string(),
            r#". . . . . . . X
. . . . . . . X
. . . . . . . X
. X X X X X X .
. . . . . . . X
. . . . . . . .
. . . . . . . .
. . . . . . . .
"#
        );
    }

    #[test]
    fn bake_magic_entries() {
        const BAKE: bool = false;
        // const BAKE: bool = true;
        if BAKE {
            use std::fs::File;
            let file = File::create("rook_magic.bin").expect("Failed to create file");
            let mut writer = BufWriter::new(file);
            for sq in 0..64 {
                let entry = bake_magic_entry_rook(Square::new(sq));

                entry.serialize(&mut writer).expect("Failed to serialize");
            }
        }
        if BAKE {
            use std::fs::File;
            let file = File::create("bishop_magic.bin").expect("Failed to create file");
            let mut writer = BufWriter::new(file);
            for sq in 0..64 {
                let entry = bake_magic_entry_bishop(Square::new(sq));

                entry.serialize(&mut writer).expect("Failed to serialize");
            }
        }
    }
}
