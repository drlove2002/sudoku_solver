use std::fmt;

use crate::types::masks::DirtyMask;

#[derive(Debug, Clone)]
pub struct PermutationNode<const N: usize, const K: usize> {
    cells: [u8; N],
    pub row_masks: [DirtyMask<N>; K],
    pub col_masks: [DirtyMask<N>; K],
    pub compatible: Vec<(usize, usize)>, // (Minigrid id, Permutation id)
}

impl<const N: usize, const K: usize> fmt::Display for PermutationNode<N, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, val) in self.cells.iter().enumerate() {
            if i > 0 {
                if i % K == 0 {
                    write!(f, " | ")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "{}", val)?;
        }
        write!(f, "]")
    }
}

impl<const N: usize, const K: usize> PermutationNode<N, K> {
    pub fn cells(&self) -> &[u8; N] {
        &self.cells
    }
    pub fn from_minigrid(cells: [u8; N]) -> Self {
        let mut row_masks = [DirtyMask::default(); K];
        let mut col_masks = [DirtyMask::default(); K];

        for (i, &digit) in cells.iter().enumerate() {
            let r = i / K; // Row
            let c = i % K; // Column
            let digit = digit as usize;
            row_masks[r].dirty_set(digit);
            col_masks[c].dirty_set(digit);
        }

        Self {
            cells,
            row_masks,
            col_masks,
            compatible: Vec::new(),
        }
    }

    /// Row-compatibility: for each of the K rows inside the KxK minigrid,
    /// the corresponding row masks must not have any overlapping digit bits.
    pub fn check_row_compatible(&self, other: &Self) -> bool {
        for c in 0..K {
            if self.row_masks[c].is_conflicting(&other.row_masks[c]) {
                return false;
            }
        }
        true
    }

    /// Column-compatibility: for each of the K columns inside the minigrid,
    /// the corresponding column masks must not overlap.
    pub fn check_col_compatible(&self, other: &Self) -> bool {
        for c in 0..K {
            if self.col_masks[c].is_conflicting(&other.col_masks[c]) {
                return false;
            }
        }
        true
    }
}
