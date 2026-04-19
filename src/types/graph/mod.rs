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
