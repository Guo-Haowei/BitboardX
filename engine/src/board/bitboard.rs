use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::Not;

#[derive(Copy, Clone)]
pub struct BitBoard {
    val: u64,
}

impl BitBoard {
    pub const fn zero() -> Self {
        Self { val: 0u64 }
    }

    pub const fn new(val: u64) -> Self {
        Self { val }
    }

    pub const fn is_empty(&self) -> bool {
        self.val == 0
    }

    pub const fn get(&self) -> u64 {
        self.val
    }

    pub fn has_bit(&self, square: u8) -> bool {
        assert!(square < 64);
        (self.val & (1u64 << square)) != 0
    }

    pub fn set_bit(&mut self, square: u8) {
        assert!(square < 64);
        self.val |= 1u64 << square;
    }

    pub fn unset_bit(&mut self, square: u8) {
        assert!(square < 64);
        self.val &= !(1u64 << square);
    }

    pub const fn equal(&self, val: u64) -> bool {
        self.val == val
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> BitBoard {
        BitBoard::new(self.val & rhs.val)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> BitBoard {
        BitBoard::new(self.val | rhs.val)
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
        BitBoard::new(!self.val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard() {
        let mut bb = BitBoard::zero();
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

        let bb2 = BitBoard::new(1u64 << 1);
        let bb3 = BitBoard::new(1u64 << 2);
        let bb4 = bb2 | bb3;
        assert!(bb4.has_bit(1));
        assert!(bb4.has_bit(2));
    }
}
