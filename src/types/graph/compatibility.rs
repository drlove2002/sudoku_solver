use crate::types::graph::PermutationNode;

impl<const N: usize, const K: usize> PermutationNode<N, K> {
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
