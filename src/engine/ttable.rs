use static_assertions::const_assert;

use crate::core::types::Move;
use crate::core::zobrist::ZobristHash;

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
    pub key: ZobristHash,        // 8 bytes
    pub score: i32,              // 4 bytes
    pub best_move: Option<Move>, // 2 bytes
    pub depth: u8,               // 1 byte, means ply searched for this entry
    pub node_type: NodeType,     // 1 byte
}

const_assert!(std::mem::size_of::<TTEntry>() == 16);
const_assert!(std::mem::size_of::<Option<TTEntry>>() == 16);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TTStoreResult {
    NotUpdated,
    UpdatedEmpty,         // entry was empty
    OverridenCollision,   // entry with the same hash, different key already exists
    OverridenNoCollision, // entry with the same hash, same key already exists
}

pub struct TranspositionTable<const N: usize> {
    table: Box<[Option<TTEntry>]>,
    count: u64,

    pub collision_count: u64,
}

impl<const N: usize> TranspositionTable<N> {
    pub fn new() -> Self {
        let data = vec![None; N];
        let table = data.into_boxed_slice();
        Self { table, count: 0, collision_count: 0 }
    }

    #[inline(always)]
    fn index(key: ZobristHash) -> usize {
        (key.0 as usize) & (N - 1)
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn store(
        &mut self,
        key: ZobristHash,
        depth: u8,
        score: i32,
        node_type: NodeType,
        best_move: Option<Move>,
    ) -> TTStoreResult {
        assert!(best_move.is_some());

        let idx = Self::index(key);
        let existing = &self.table[idx];

        let result = if let Some(old_entry) = existing {
            if old_entry.key != key {
                self.collision_count += 1;
                TTStoreResult::OverridenCollision
            } else if depth >= old_entry.depth {
                TTStoreResult::OverridenNoCollision
            } else {
                TTStoreResult::NotUpdated
            }
        } else {
            self.count += 1; // only increment count if we are inserting a new entry
            TTStoreResult::UpdatedEmpty
        };

        if result != TTStoreResult::NotUpdated {
            self.table[idx] = Some(TTEntry { key, depth, score, node_type, best_move });
        }

        result
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
