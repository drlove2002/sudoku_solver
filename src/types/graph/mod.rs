mod compatibility;
mod node;
mod relationship;
mod visualize;

use log::trace;
pub use node::PermutationNode;
pub use relationship::Relation;

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

    pub fn create_edges(&mut self) {
        for i in 0..N {
            for j in (i + 1)..N {
                // Compute relationship BEFORE borrowing minigrids mutably
                let relation = self.relationship(i, j);

                let (left, right) = self.minigrids.split_at_mut(j);
                let mgi = &mut left[i];
                let mgj = &mut right[0];

                for (pi_idx, pi) in mgi.iter_mut().enumerate() {
                    for (pj_idx, pj) in mgj.iter_mut().enumerate() {
                        let compatible = match relation {
                            Relation::Not => false,
                            Relation::Row => pi.check_row_compatible(pj),
                            Relation::Col => pi.check_col_compatible(pj),
                        };

                        trace!(
                            "{pi_idx}-{i} and {pi_idx}-{j} are {:?} compatible",
                            relation
                        );
                        if compatible {
                            pi.compatible.push((j, pj_idx));
                            pj.compatible.push((i, pi_idx));
                        }
                    }
                }
            }
        }
    }
}
