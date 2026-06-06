use std::{
    fmt,
    ops::{BitAnd, BitOr, BitOrAssign},
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BitString<const N: usize> {
    bits: u32,
}

impl<const N: usize> BitString<N> {
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.bits
    }

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
impl<const N: usize> BitAnd for BitString<N> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitString {
            bits: self.bits & rhs.bits,
        }
    }
}

impl<const N: usize> BitOrAssign for BitString<N> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

pub type EmptyMask<const N: usize> = BitString<N>;
impl<const N: usize> EmptyMask<N> {
    #[inline(always)]
    pub fn set_value(&mut self, idx: usize, value: u8) {
        self.bits |= ((value == 0) as u32) << idx;
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

pub type DirtyMask<const N: usize> = BitString<N>;
impl<const N: usize> DirtyMask<N> {
    #[inline(always)]
    pub fn is_dirty(&self, num: usize) -> bool {
        self.is_set(num - 1)
    }

    #[inline(always)]
    pub fn dirty_set(&mut self, num: usize) -> &Self {
        self.set(num - 1)
    }

    #[inline(always)]
    pub fn is_conflicting(&self, rhs: &Self) -> bool {
        self.bits & rhs.bits != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicBitSet {
    words: Vec<u64>,
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedBitSet {
    words: Box<[u64]>,
    len: usize,
}

impl DynamicBitSet {
    pub fn new(len: usize) -> Self {
        let word_count = len.div_ceil(64);
        Self {
            words: vec![0; word_count],
            len,
        }
    }

    pub fn full(len: usize) -> Self {
        let mut bitset = Self {
            words: vec![u64::MAX; len.div_ceil(64)],
            len,
        };
        bitset.mask_unused_bits();
        bitset
    }

    pub fn singleton(len: usize, idx: usize) -> Self {
        let mut bitset = Self::new(len);
        bitset.set(idx);
        bitset
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&word| word == 0)
    }

    pub fn count_ones(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    pub fn set(&mut self, idx: usize) {
        debug_assert!(idx < self.len);
        self.words[idx / 64] |= 1u64 << (idx % 64);
    }

    pub fn contains(&self, idx: usize) -> bool {
        debug_assert!(idx < self.len);
        (self.words[idx / 64] & (1u64 << (idx % 64))) != 0
    }

    pub fn intersect_with(&mut self, other: &Self) -> bool {
        debug_assert_eq!(self.len, other.len);
        let mut changed = false;

        for (left, right) in self.words.iter_mut().zip(other.words.iter()) {
            let next = *left & *right;
            changed |= next != *left;
            *left = next;
        }

        changed
    }

    pub fn intersect_with_fixed(&mut self, other: &FixedBitSet) -> bool {
        debug_assert_eq!(self.len, other.len);
        let mut changed = false;

        for (left, right) in self.words.iter_mut().zip(other.words.iter()) {
            let next = *left & *right;
            changed |= next != *left;
            *left = next;
        }

        changed
    }

    pub fn iter_ones(&self) -> DynamicBitSetIter<'_> {
        DynamicBitSetIter {
            words: &self.words,
            word_idx: 0,
            current_word: self.words.first().copied().unwrap_or(0),
            base_idx: 0,
            len: self.len,
        }
    }

    fn mask_unused_bits(&mut self) {
        let remainder = self.len % 64;
        if remainder == 0 || self.words.is_empty() {
            return;
        }

        let mask = (1u64 << remainder) - 1;
        let last = self.words.len() - 1;
        self.words[last] &= mask;
    }
}

impl FixedBitSet {
    pub fn new(len: usize) -> Self {
        let word_count = len.div_ceil(64);
        Self {
            words: vec![0; word_count].into_boxed_slice(),
            len,
        }
    }

    pub fn from_words(words: Vec<u64>, len: usize) -> Self {
        let mut bitset = Self {
            words: words.into_boxed_slice(),
            len,
        };
        bitset.mask_unused_bits();
        bitset
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&word| word == 0)
    }

    pub fn count_ones(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    pub fn contains(&self, idx: usize) -> bool {
        debug_assert!(idx < self.len);
        (self.words[idx / 64] & (1u64 << (idx % 64))) != 0
    }

    pub fn iter_ones(&self) -> DynamicBitSetIter<'_> {
        DynamicBitSetIter {
            words: &self.words,
            word_idx: 0,
            current_word: self.words.first().copied().unwrap_or(0),
            base_idx: 0,
            len: self.len,
        }
    }

    pub fn iter(&self) -> &[u64] {
        &self.words
    }

    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.words.len() * 8
    }

    fn mask_unused_bits(&mut self) {
        let remainder = self.len % 64;
        if remainder == 0 || self.words.is_empty() {
            return;
        }

        let mask = (1u64 << remainder) - 1;
        let last = self.words.len() - 1;
        self.words[last] &= mask;
    }
}

pub struct DynamicBitSetIter<'a> {
    words: &'a [u64],
    word_idx: usize,
    current_word: u64,
    base_idx: usize,
    len: usize,
}

impl Iterator for DynamicBitSetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_word != 0 {
                let bit = self.current_word.trailing_zeros() as usize;
                self.current_word &= self.current_word - 1;
                let idx = self.base_idx + bit;
                return (idx < self.len).then_some(idx);
            }

            self.word_idx += 1;
            if self.word_idx >= self.words.len() {
                return None;
            }

            self.base_idx = self.word_idx * 64;
            self.current_word = self.words[self.word_idx];
        }
    }
}
