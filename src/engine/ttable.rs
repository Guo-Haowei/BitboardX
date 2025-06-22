use static_assertions::const_assert;

use crate::core::types::Move;
use crate::core::zobrist::ZobristHash;

macro_rules! tt_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

#[derive(Debug, Copy, Clone)]
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
    pub key: ZobristHash,    // 8 bytes
    pub score: i32,          // 4 bytes
    pub best_move: Move,     // 2 bytes
    pub depth: u8,           // 1 byte, means ply searched for this entry
    pub node_type: NodeType, // 1 byte
}

impl TTEntry {
    pub const fn empty() -> Self {
        let key = ZobristHash(0);
        Self { key, depth: 0, node_type: NodeType::None, score: 0, best_move: Move::null() }
    }
}

pub struct TranspositionTable<const N: usize> {
    table: Box<[TTEntry]>,
}

impl<const N: usize> TranspositionTable<N> {
    pub fn new() -> Self {
        let data = vec![TTEntry::empty(); N];
        let table = data.into_boxed_slice();
        Self { table }
    }

    #[inline(always)]
    fn index(key: ZobristHash) -> usize {
        (key.0 as usize) & (N - 1)
    }

    pub fn store(
        &mut self,
        key: ZobristHash,
        depth: u8,
        score: i32,
        node_type: NodeType,
        best_move: Move,
    ) {
        let idx = Self::index(key);
        let existing = &self.table[idx];

        if existing.key == ZobristHash::null() || depth >= existing.depth {
            tt_debug!(
                "Storing TTEntry: key: {:?}, depth: {}, score: {}, node_type: {:?}, best_move: {}",
                key,
                depth,
                score,
                node_type,
                best_move.to_string()
            );
            self.table[idx] = TTEntry { key, depth, score, node_type, best_move };
        }
    }

    pub fn probe(&self, key: ZobristHash) -> Option<&TTEntry> {
        assert!(key.0 != 0, "ZobristHash cannot be zero");
        let idx = Self::index(key);
        let entry = &self.table[idx];

        if entry.key == key { Some(entry) } else { None }
    }

    pub fn clear(&mut self) {
        self.table.fill(TTEntry::empty());
    }
}

const DEFAULT_TT_SIZE_IN_BYTE: usize = 16 * 1024 * 1024; // 32 MB

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
