use crate::types::{
    Minigrid,
    bitstring::DirtyMask,
    graph::{GeneratedMinigrid, PermId, PermutationNode},
    masks::Masks,
};
use log::{debug, info, trace};
use rayon::prelude::*;

pub struct PermutationGenerator<'a, const N: usize, const K: usize> {
    mg: Minigrid<N, K>,
    masks: &'a Masks<N>,
    nodes: Vec<PermutationNode<N, K>>,
    payloads: Vec<[u8; N]>,
}

impl<'a, const N: usize, const K: usize> PermutationGenerator<'a, N, K> {
    pub fn new(mg: Minigrid<N, K>, masks: &'a Masks<N>) -> Self {
        Self {
            mg,
            masks,
            nodes: Vec::new(),
            payloads: Vec::new(),
        }
    }

    pub fn generate(mut self) -> GeneratedMinigrid<N, K> {
        let used_mask = self.masks.boxs[self.mg.id];
        info!(
            "Generating permutations for Minigrid {} (initial_mask={})",
            self.mg.id, used_mask
        );

        self.dfs(used_mask);

        info!(
            "Minigrid {}: {} permutation(s)",
            self.mg.id,
            self.nodes.len()
        );
        GeneratedMinigrid {
            nodes: self.nodes,
            payloads: self.payloads,
        }
    }

    // Select the empty cell with the fewest candidates (MRV heuristic)
    // Returns Some(index) of the best cell, or None if no empty cells are found
    // MRV: Minimum Remaining Values
    #[inline(always)]
    fn find_best_cell(&self, used_mask: DirtyMask<N>) -> Option<(usize, DirtyMask<N>)> {
        let start_row = (self.mg.id / K) * K;
        let start_col = (self.mg.id % K) * K;
        let mut best_idx = None;
        let mut best_count = 0;

        trace!("Empty Mask: {}, UsedMask: {}", self.mg.empty, used_mask);
        for idx in self.mg.empty {
            if self.mg.cells[idx] != 0 {
                unreachable!("Already handeled by empty_mask")
            }

            let global_row = start_row + (idx / K);
            let global_col = start_col + (idx % K);

            let mut conflict = self.masks.conflict[global_row][global_col];
            trace!("Conflict Mask: {}", conflict);
            conflict |= used_mask;

            if conflict.is_all_set() {
                trace!(
                    "  Cell[{}] impossible at ({},{})",
                    idx, global_row, global_col
                );
                return None;
            }

            let incompatible_candidate_count = conflict.get().count_ones();
            trace!(
                "Try Cell[{}] Pos({},{}) InvalidCandidates:{} Mask:{}",
                idx, global_row, global_col, incompatible_candidate_count, conflict
            );
            if incompatible_candidate_count > best_count {
                // Less candidates found, update best choice
                best_count = incompatible_candidate_count;
                best_idx = Some((idx, conflict));
                trace!("Set Cell[{}] Pos({},{})", idx, global_row, global_col);
                if best_count == ((N - 1) as u32) {
                    // For 9x9, already 8 candidates are set, 1 remaining
                    // This must be the best choice, we can early break
                    break;
                }
            }
        }

        debug!(
            "find_best_cell(mg={}, UsedMask={}): {:?}",
            self.mg.id, used_mask, best_idx
        );
        best_idx
    }

    fn dfs(&mut self, used_mask: DirtyMask<N>) {
        if let Some((current_idx, conflict)) = self.find_best_cell(used_mask) {
            let available = (!conflict.get()) & ((1u32 << N) - 1);
            let mut candidates = available;

            while candidates != 0 {
                let num = candidates.trailing_zeros() as usize + 1;
                candidates &= candidates - 1;

                trace!("  Try num={}", num);
                self.mg.cells[current_idx] = num as u8;
                self.mg.empty.reset(current_idx);

                let mut next_mask = used_mask;
                next_mask.dirty_set(num);

                self.dfs(next_mask);

                // Backtrack
                self.mg.cells[current_idx] = 0;
                self.mg.empty.set(current_idx);
            }
        } else if used_mask.is_all_set() {
            trace!("✓ Solution found for mg={}", self.mg.id);
            let cells = self.mg.cells;
            let payload_idx =
                PermId::try_from(self.payloads.len()).expect("payload count must fit into u32");
            self.payloads.push(cells);
            self.nodes
                .push(PermutationNode::from_minigrid(&cells, payload_idx));
        } else {
            trace!(
                "✗ Dead end at MinigridIdx={}, Mask={}",
                self.mg.id, used_mask
            );
        }
    }
}

pub fn generate_all_permutations<const N: usize, const K: usize>(
    board: &crate::types::Board<N>,
    masks: &Masks<N>,
) -> [GeneratedMinigrid<N, K>; N] {
    debug!(
        "Starting parallel permutation generation for {} minigrid(s)",
        N
    );

    (0..N)
        .into_par_iter()
        .map(|id| {
            let mg = Minigrid::new(id, board);
            PermutationGenerator::new(mg, masks).generate()
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
