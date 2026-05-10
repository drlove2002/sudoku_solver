use crate::types::bitstring::DirtyMask;

pub type PermId = u32;

#[derive(Debug, Clone)]
pub struct PermutationNode<const N: usize, const K: usize> {
    payload_idx: PermId,
    pub row_masks: [DirtyMask<N>; K],
    pub col_masks: [DirtyMask<N>; K],
}

impl<const N: usize, const K: usize> PermutationNode<N, K> {
    pub fn payload_idx(&self) -> PermId {
        self.payload_idx
    }

    pub fn set_payload_idx(&mut self, payload_idx: PermId) {
        self.payload_idx = payload_idx;
    }

    pub fn from_minigrid(cells: &[u8; N], payload_idx: PermId) -> Self {
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
            payload_idx,
            row_masks,
            col_masks,
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
