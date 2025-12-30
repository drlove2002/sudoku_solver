use std::{
    fmt,
    ops::{BitOr, BitOrAssign},
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BitString<const N: usize> {
    bits: u32,
}

impl<const N: usize> BitString<N> {
    pub fn is_all_set(&self) -> bool {
        // Check if all bits from.bits to N-1 are set
        // example: N=4 ->.bitsb0000_1111 = (1 << 4) - 1 = 15
        self.bits == (1 << N) - 1
    }

    // For logical deduction (mark unit dirty)
    #[inline(always)]
    pub fn set(&mut self, idx: usize) -> &Self {
        self.bits |= 1u32 << idx;
        self
    }

    // For logical deduction (mark unit clean)
    #[inline(always)]
    pub fn reset(&mut self, idx: usize) {
        self.bits &= !(1u32 << idx);
    }

    pub fn get(&self) -> &u32 {
        &self.bits
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    #[inline(always)]
    pub fn is_set(&self, i: usize) -> bool {
        (self.bits & (1u32 << i)) != 0
    }
}

impl<const N: usize> fmt::Display for BitString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$b}", self.bits, width = N)
    }
}

impl<const N: usize> BitOr for BitString<N> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        BitString {
            bits: self.bits | rhs.bits,
        }
    }
}

impl<const N: usize> BitOrAssign for BitString<N> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

pub type EmptyMask<const N: usize> = BitString<N>;
pub type DirtyMask<const N: usize> = BitString<N>;

impl<const N: usize> EmptyMask<N> {
    #[inline(always)]
    pub fn set_value(&mut self, idx: usize, value: u8) {
        self.bits |= ((value == 0) as u32) << idx;
    }
}

impl<const N: usize> DirtyMask<N> {
    #[inline(always)]
    pub fn is_dirty(&self, num: usize) -> bool {
        self.is_set(num - 1)
    }

    #[inline(always)]
    pub fn dirty_set(&mut self, num: usize) -> &Self {
        self.set(num - 1)
    }
}

/// Iterator using Kernighan's trick
impl<const N: usize> Iterator for EmptyMask<N> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.bits == 0 {
            None
        } else {
            let idx = self.bits.trailing_zeros() as usize;
            self.bits &= self.bits - 1; // Brian Kernighan's trick
            Some(idx)
        }
    }
}
