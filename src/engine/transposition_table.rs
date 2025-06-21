use crate::core::types::Move;
use crate::core::zobrist;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    zobrist: zobrist::ZobristHash,
    ply: u8,
    node_type: NodeType,
    score: i32,
    best_move: Move, // Usually packed as a u16
}
