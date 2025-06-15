use super::super::board::BitBoard;

pub const NORTH: i32 = 8;
pub const SOUTH: i32 = -NORTH;
pub const EAST: i32 = 1;
pub const WEST: i32 = -EAST;
pub const NE: i32 = NORTH + EAST;
pub const NW: i32 = NORTH + WEST;
pub const SE: i32 = SOUTH + EAST;
pub const SW: i32 = SOUTH + WEST;

pub const BOUND_A: BitBoard = BitBoard::from(0x0101010101010101);
pub const BOUND_B: BitBoard = BitBoard::from(0x0202020202020202);
pub const BOUND_G: BitBoard = BitBoard::from(0x4040404040404040);
pub const BOUND_H: BitBoard = BitBoard::from(0x8080808080808080);
pub const BOUND_1: BitBoard = BitBoard::from(0x00000000000000FF);
pub const BOUND_2: BitBoard = BitBoard::from(0x000000000000FF00);
pub const BOUND_7: BitBoard = BitBoard::from(0x00FF000000000000);
pub const BOUND_8: BitBoard = BitBoard::from(0xFF00000000000000);
pub const BOUND_AB: BitBoard = BitBoard::from(BOUND_A.get() | BOUND_B.get());
pub const BOUND_GH: BitBoard = BitBoard::from(BOUND_G.get() | BOUND_H.get());
pub const BOUND_12: BitBoard = BitBoard::from(BOUND_1.get() | BOUND_2.get());
pub const BOUND_78: BitBoard = BitBoard::from(BOUND_7.get() | BOUND_8.get());

pub fn shift(bb: BitBoard, dir: i32) -> BitBoard {
    // if dir > 0 { bb.get() << dir } else { bb.get() >> -dir }
    BitBoard::from(if dir < 0 { bb.get() >> -dir } else { bb.get() << dir })
}

pub fn shift_east(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_H).shift(EAST)
}

pub fn shift_west(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_A).shift(WEST)
}

pub fn shift_north(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_8).shift(NORTH)
}

pub fn shift_south(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_1).shift(SOUTH)
}

pub fn shift_ne(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_H | BOUND_8)).shift(NE)
}

pub fn shift_nw(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_A | BOUND_8)).shift(NW)
}

pub fn shift_se(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_H | BOUND_1)).shift(SE)
}

pub fn shift_sw(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_A | BOUND_1)).shift(SW)
}

pub const SHIFT_FUNCS: [fn(BitBoard) -> BitBoard; 8] =
    [shift_north, shift_south, shift_east, shift_west, shift_ne, shift_nw, shift_se, shift_sw];
