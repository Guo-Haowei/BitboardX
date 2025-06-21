use static_assertions::const_assert;

use crate::core::types::Move;
use crate::core::zobrist::ZobristHash;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum NodeType {
    None = 0,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TTEntry {
    key: ZobristHash,    // 8 bytes
    score: i32,          // 4 bytes
    best_move: Move,     // 2 bytes
    ply: u8,             // 1 byte
    node_type: NodeType, // 1 byte
}

impl TTEntry {
    pub const fn empty() -> Self {
        let key = ZobristHash(0);
        Self { key, ply: 0, node_type: NodeType::None, score: 0, best_move: Move::null() }
    }
}

pub struct TranspositionTable<const N: usize> {
    table: Box<[TTEntry; N]>,
}

impl<const N: usize> TranspositionTable<N> {
    pub fn new() -> Self {
        let table = Box::new([TTEntry::empty(); N]);
        Self { table }
    }

    #[inline(always)]
    fn index(key: ZobristHash) -> usize {
        (key.0 as usize) & (N - 1)
    }

    pub fn store(
        &mut self,
        key: ZobristHash,
        ply: u8,
        score: i32,
        node_type: NodeType,
        best_move: Move,
    ) {
        let idx = Self::index(key);
        let existing = &self.table[idx];

        if existing.key == ZobristHash::null() || ply >= existing.ply {
            self.table[idx] = TTEntry { key, ply, score, node_type, best_move };
        }
    }

    pub fn probe(&self, key: ZobristHash) -> Option<&TTEntry> {
        let idx = Self::index(key);
        let entry = &self.table[idx];

        if entry.key == key { Some(entry) } else { None }
    }

    pub fn clear(&mut self) {
        self.table.fill(TTEntry::empty());
    }
}

const DEFAULT_TT_SIZE_IN_BYTE: usize = 32 * 1024; // 32 KB
// const DEFAULT_TT_SIZE_IN_BYTE: usize = 32 * 1024 * 1024; // 32 MB

pub type TTable = TranspositionTable<{ DEFAULT_TT_SIZE_IN_BYTE / std::mem::size_of::<TTEntry>() }>;

const_assert!(std::mem::size_of::<TTEntry>() == 16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_entry_size() {
        assert_eq!(std::mem::size_of::<TTEntry>(), 16);
    }

    #[test]
    fn test_tt_entry_empty() {
        let entry = TTEntry::empty();
        let ptr = &entry as *const TTEntry as *const u64;
        let slice: &[u64] = unsafe { std::slice::from_raw_parts(ptr, 2) };
        assert_eq!(slice[0], 0);
        assert_eq!(slice[1], 0);
    }
}
