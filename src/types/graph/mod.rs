mod node;
mod visualize;

use log::trace;
pub use node::PermutationNode;

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

    pub fn create_edges(&mut self) {
        use log::info;
        let mut total_edges = 0;

        for i in 0..N {
            for j in (i + 1)..N {
                // Compute relationship BEFORE borrowing minigrids mutably
                let relation = self.relationship(i, j);

                let (left, right) = self.minigrids.split_at_mut(j);
                let mgi = &mut left[i];
                let mgj = &mut right[0];

                let mut edges_added = 0;

                for (pi_idx, pi) in mgi.iter_mut().enumerate() {
                    for (pj_idx, pj) in mgj.iter_mut().enumerate() {
                        let compatible = match relation {
                            Relation::Not => false,
                            Relation::Row => pi.check_row_compatible(pj),
                            Relation::Col => pi.check_col_compatible(pj),
                        };

                        trace!(
                            "{pi_idx}-{i} and {pj_idx}-{j} are {:?} compatible: {}",
                            relation, compatible
                        );
                        if compatible {
                            pi.compatible.push((j, pj_idx));
                            pj.compatible.push((i, pi_idx));
                            edges_added += 1;
                        }
                    }
                }

                info!(
                    "MG{}-MG{} ({:?}): {} edge(s) added",
                    i, j, relation, edges_added
                );
                total_edges += edges_added;
            }
        }

        info!("Total edges created: {}", total_edges);
    }

    /// Retain only the requested permutation indices per minigrid and remap
    /// compatibility edges to the new dense index space.
    pub fn retain_permutations(&mut self, keep: &[Vec<usize>; N]) {
        let old = std::mem::replace(&mut self.minigrids, std::array::from_fn(|_| Vec::new()));

        let remap: [Vec<Option<usize>>; N] = std::array::from_fn(|mg_id| {
            let mut mapping = vec![None; old[mg_id].len()];
            for (new_idx, &old_idx) in keep[mg_id].iter().enumerate() {
                mapping[old_idx] = Some(new_idx);
            }
            mapping
        });

        self.minigrids = std::array::from_fn(|mg_id| {
            keep[mg_id]
                .iter()
                .map(|&old_idx| {
                    let mut node = old[mg_id][old_idx].clone();
                    node.compatible = node
                        .compatible
                        .iter()
                        .filter_map(|&(target_mg, target_perm)| {
                            remap[target_mg][target_perm]
                                .map(|new_target_perm| (target_mg, new_target_perm))
                        })
                        .collect();
                    node
                })
                .collect()
        });
    }

    /// Get the degrees of all permutations in a minigrid
    /// Returns an iterator of (permutation_id, degree) pairs
    pub fn permutation_degrees(&self, mg_id: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.minigrids[mg_id]
            .iter()
            .enumerate()
            .map(|(idx, node)| (idx, node.compatible.len()))
    }

    /// Remove a permutation from a minigrid
    pub fn remove_permutation(&mut self, mg_id: usize, perm_id: usize) {
        self.minigrids[mg_id].remove(perm_id);
    }

    /// Clean up edges pointing to removed permutations
    /// This updates all edge lists to remove references to permutations that no longer exist
    pub fn cleanup_edges(&mut self) {
        // First, collect the lengths of all minigrids to avoid borrow checker issues
        let lengths: [usize; N] = std::array::from_fn(|i| self.minigrids[i].len());

        // Now update edges
        for mg_id in 0..N {
            for node in &mut self.minigrids[mg_id] {
                node.compatible.retain(|(target_mg, target_perm)| {
                    // Keep edge only if target permutation still exists
                    *target_perm < lengths[*target_mg]
                });
            }
        }
    }

    /// Count total permutations across all minigrids
    pub fn total_permutations(&self) -> usize {
        self.minigrids.iter().map(|mg| mg.len()).sum()
    }

    /// Count undirected compatibility edges across all minigrids.
    pub fn total_edges(&self) -> usize {
        self.minigrids
            .iter()
            .flat_map(|mg| mg.iter())
            .map(|node| node.compatible.len())
            .sum::<usize>()
            / 2
    }

    /// Count permutations in a specific minigrid
    pub fn permutation_count(&self, mg_id: usize) -> usize {
        self.minigrids[mg_id].len()
    }

    /// Get the degree (number of compatible edges) for a specific permutation
    pub fn permutation_degree(&self, mg_id: usize, perm_id: usize) -> usize {
        self.minigrids[mg_id][perm_id].compatible.len()
    }

    /// Get the compatible edges for a specific permutation
    pub fn compatible_edges(&self, mg_id: usize, perm_id: usize) -> &[(usize, usize)] {
        &self.minigrids[mg_id][perm_id].compatible
    }

    /// Get the cell values for a specific permutation
    pub fn permutation_cells(&self, mg_id: usize, perm_id: usize) -> &[u8; N] {
        self.minigrids[mg_id][perm_id].cells()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
