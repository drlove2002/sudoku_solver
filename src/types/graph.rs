use std::fmt;

use crate::types::masks::DirtyMask;

#[derive(Debug)]
pub struct PermutationNode<const N: usize, const K: usize> {
    cells: [u8; N],
    pub row_masks: [DirtyMask<N>; K],
    pub col_masks: [DirtyMask<N>; K],
    pub compatible: Vec<u8>,
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
}

/// Graph structure for storing PermutationNodes and their compatibility edges
pub struct Graph<const K: usize, const N: usize> {
    /// Array of PermutationNode vectors, one per minigrid
    minigrids: [Vec<PermutationNode<N, K>>; N],
}

impl<const K: usize, const N: usize> Graph<K, N> {
    /// Initialize graph from permutation data and build compatibility edges
    pub fn new(minigrids: [Vec<PermutationNode<N, K>>; N]) -> Self {
        Self { minigrids }
    }
}
