use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BitBoard {
    val: u64,
}

impl BitBoard {
    pub const fn new() -> Self {
        Self { val: 0u64 }
    }

    pub const fn from(val: u64) -> Self {
        Self { val }
    }

    pub const fn from_bit(bit: u8) -> Self {
        Self { val: 1u64 << bit }
    }

    pub const fn is_empty(&self) -> bool {
        self.val == 0
    }

    pub const fn get(&self) -> u64 {
        self.val
    }

    pub const fn has_bit(&self, bit: u8) -> bool {
        (self.val & (1u64 << bit)) != 0
    }

    pub const fn set_bit(&mut self, bit: u8) {
        self.val |= 1u64 << bit;
    }

    pub const fn unset_bit(&mut self, bit: u8) {
        self.val &= !(1u64 << bit);
    }

    pub const fn equal(&self, val: u64) -> bool {
        self.val == val
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> BitBoard {
        BitBoard::from(self.val & rhs.val)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.val &= rhs.val;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> BitBoard {
        BitBoard::from(self.val | rhs.val)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.val |= rhs.val;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> BitBoard {
        BitBoard::from(!self.val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard() {
        let mut bb = BitBoard::new();
        assert!(!bb.has_bit(0));
        assert!(!bb.has_bit(63));

        bb.set_bit(0);
        assert!(bb.has_bit(0));
        assert!(!bb.has_bit(1));

        bb.set_bit(63);
        assert!(bb.has_bit(63));
        assert!(!bb.has_bit(62));

        bb.unset_bit(0);
        assert!(!bb.has_bit(0));
        assert!(bb.has_bit(63));

        let bb2 = BitBoard::from(1u64 << 1);
        let bb3 = BitBoard::from(1u64 << 2);
        let bb4 = bb2 | bb3;
        assert!(bb4.has_bit(1));
        assert!(bb4.has_bit(2));
    }
}
