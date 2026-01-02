//! Provides `Relation` and the `relation` method implementation for `Graph`
//! (implemented as an `impl` on the `Graph` type defined in the parent module).

/// Compatibility relation between two minigrids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// Same block-row (e.g., indices 0 and 1 in a 3x3 block grid)
    Row,
    /// Same block-column (e.g., indices 0 and 3 in a 3x3 block grid)
    Col,
    /// Not compatible (includes the same-index case per your mapping)
    Not,
}

impl Relation {
    /// Convert a 2-bit mask (0..=3) to a `Relation` variant.
    ///
    /// Mask bit layout (bit0 = row_eq, bit1 = col_eq):
    ///  - 0b00 -> 0 -> Not
    ///  - 0b01 -> 1 -> Row
    ///  - 0b10 -> 2 -> Col
    ///  - 0b11 -> 3 -> Not  (same block => treated as Not per your request)
    #[inline]
    pub fn from_mask(mask: usize) -> Self {
        const LUT: [Relation; 4] = [
            Relation::Not, // 0b00
            Relation::Row, // 0b01
            Relation::Col, // 0b10
            Relation::Not, // 0b11
        ];
        LUT[mask & 3]
    }
}

/// Implement the `relation` method directly on `Graph`.
///
/// Uses the `K` const parameter from the `Graph` type as the block dimension.
///
/// Strategy (branch-lean, bit-trick):
///  1. compute whether the block-rows are equal -> row_eq (0 or 1)
///  2. compute whether the block-cols are equal -> col_eq (0 or 1)
///  3. pack into mask = row_eq | (col_eq << 1)
///  4. map mask -> Relation via `Compatible::from_mask`
///
/// Notes:
///  - `K` is the block-dimension (for a 9x9 Sudoku K == 3).
///  - `a` and `b` are minigrid indices in row-major order:
///    for K = 3 (9x9):
///    0 1 2
///    3 4 5
///    6 7 8
///  - The function intentionally does NOT validate input ranges (as requested).
impl<const K: usize, const N: usize> super::Graph<K, N> {
    /// Determine compatibility relation between minigrid index `a` and `b`.
    ///
    /// Example (K = 3, 9x9):
    ///  - relation(3, 0, 1) => Row
    ///    row: 0/3=0, 1/3=0 -> row_eq = 1
    ///    col: 0%3=0, 1%3=1 -> col_eq = 0
    ///    mask = 1 -> Relation::Row
    ///
    ///  - relation(3, 0, 3) => Col
    ///    row: 0/3=0, 3/3=1 -> row_eq = 0
    ///    col: 0%3=0, 3%3=0 -> col_eq = 1
    ///    mask = 2 -> Relation::Col
    ///
    ///  - relation(3, 4, 4) => Not (same index -> Not per mapping)
    ///    row_eq = 1, col_eq = 1 -> mask = 3 -> Relation::Not
    #[inline]
    pub fn relationship(&self, a: usize, b: usize) -> Relation {
        // compute block-row equality: 1 if equal else 0
        // ((a / K) ^ (b / K)) == 0 -> true when equal
        let row_eq = (((a / K) ^ (b / K)) == 0) as usize;

        // compute block-col equality: 1 if equal else 0
        // ((a % K) ^ (b % K)) == 0 -> true when equal
        let col_eq = (((a % K) ^ (b % K)) == 0) as usize;

        // build 2-bit mask: bit0 = row_eq, bit1 = col_eq
        let mask = row_eq | (col_eq << 1);

        // convert mask to Relation
        Relation::from_mask(mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // bring parent Graph into scope (parent of compatibility module)
    use super::super::{Graph, PermutationNode};

    /// Create a minimal Graph instance for tests:
    /// We don't need any PermutationNodes for testing `relation`, so fill with empty Vecs.
    fn make_graph<const K: usize, const N: usize>() -> Graph<K, N> {
        // [Vec::new(); N] creates N clones of an empty Vec<PermutationNode<N, K>>
        Graph::new([const { Vec::<PermutationNode<N, K>>::new() }; N])
    }

    #[test]
    fn test_examples_9x9() {
        const K: usize = 3;
        const N: usize = K * K;
        let g = make_graph::<K, N>();

        // row-compatible examples
        assert_eq!(g.relationship(0, 1), Relation::Row);
        assert_eq!(g.relationship(1, 2), Relation::Row);
        assert_eq!(g.relationship(3, 5), Relation::Row);

        // col-compatible examples
        assert_eq!(g.relationship(0, 3), Relation::Col);
        assert_eq!(g.relationship(3, 6), Relation::Col);
        assert_eq!(g.relationship(2, 8), Relation::Col);

        // not-compatible examples (including same-index)
        assert_eq!(g.relationship(0, 4), Relation::Not);
        assert_eq!(g.relationship(2, 6), Relation::Not);
        assert_eq!(g.relationship(5, 5), Relation::Not); // same index -> Not
    }
}
