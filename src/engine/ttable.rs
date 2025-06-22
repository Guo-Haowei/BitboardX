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
    Exact = 1,
    LowerBound = 2,
    UpperBound = 3,
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

const_assert!(std::mem::size_of::<TTEntry>() == 16);
const_assert!(std::mem::size_of::<Option<TTEntry>>() == 16);

pub struct TranspositionTable<const N: usize> {
    table: Box<[Option<TTEntry>]>,
}

impl<const N: usize> TranspositionTable<N> {
    pub fn new() -> Self {
        let data = vec![None; N];
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
    ) -> bool {
        let idx = Self::index(key);
        let existing = &self.table[idx];

        if let Some(old_entry) = existing {
            // If the entry already exists, and deeper than the new one,
            // we don't want to update it
            if old_entry.key == key && old_entry.depth > depth {
                return false;
            }
        }

        tt_debug!(
            "Storing TTEntry: key: {:?}, depth: {}, score: {}, node_type: {:?}, best_move: {}",
            key,
            depth,
            score,
            node_type,
            best_move.to_string()
        );
        self.table[idx] = Some(TTEntry { key, depth, score, node_type, best_move });
        true
    }

    pub fn probe(&self, key: ZobristHash) -> Option<&TTEntry> {
        assert!(key.0 != 0, "ZobristHash cannot be zero");
        let idx = Self::index(key);
        let entry = &self.table[idx];
        match entry {
            Some(e) => {
                if e.key == key {
                    Some(e)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn clear(&mut self) {
        self.table.fill(None);
    }
}

const DEFAULT_TT_SIZE_IN_BYTE: usize = 16 * 1024 * 1024; // 32 MB

pub type TTable = TranspositionTable<{ DEFAULT_TT_SIZE_IN_BYTE / std::mem::size_of::<TTEntry>() }>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_entry_size() {
        assert_eq!(std::mem::size_of::<TTEntry>(), 16);
        assert_eq!(std::mem::size_of::<Option<TTEntry>>(), 16);
    }
}
