use crate::types::{
    Minigrid,
    graph::PermutationNode,
    masks::{DirtyMask, Masks},
};
use log::{debug, trace};
use rayon::prelude::*;

impl<const N: usize, const K: usize> Minigrid<N, K> {
    // Select the empty cell with the fewest candidates (MRV heuristic)
    // Returns Some(index) of the best cell, or None if no empty cells are found
    // MRV: Minimum Remaining Values
    #[inline(always)]
    fn find_best_cell(
        &self,
        used_mask: &DirtyMask<N>,
        masks: &Masks<N>,
    ) -> Option<(usize, DirtyMask<N>)> {
        let start_row = (self.id / Self::K) * Self::K;
        let start_col = (self.id % Self::K) * Self::K;
        let mut best_idx = None;
        let mut best_count = 0;

        trace!("Empty Mask: {}, UsedMask: {}", self.empty, used_mask);
        for idx in self.empty {
            if self.cells[idx] != 0 {
                unreachable!("Already handeled by empty_mask")
            }

            let global_row = start_row + (idx / Self::K);
            let global_col = start_col + (idx % Self::K);

            let mut conflict = masks.conflict[global_row][global_col];
            trace!("Conflict Mask: {}", conflict);
            conflict |= *used_mask;

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
            self.id, used_mask, best_idx
        );
        best_idx
    }

    fn generate_permutations_dfs(
        &mut self,
        used_mask: &DirtyMask<N>,
        masks: &Masks<N>,
        results: &mut Vec<PermutationNode<N, K>>,
    ) {
        if let Some((current_idx, conflict)) = self.find_best_cell(used_mask, masks) {
            for num in 1..=N {
                // Check if num can be placed
                if !conflict.is_dirty(num) {
                    trace!("  Try num={}", num);
                    self.cells[current_idx] = num as u8;
                    self.empty.reset(current_idx);
                    self.generate_permutations_dfs(
                        used_mask.clone().dirty_set(num),
                        masks,
                        results,
                    );
                    // Backtrack
                    self.cells[current_idx] = 0;
                    self.empty.set(current_idx);
                }
            }
        } else if used_mask.is_all_set() {
            trace!("✓ Solution found for mg={}", self.id);
            let cells = self.cells;
            results.push(PermutationNode::from_minigrid(cells));
        } else {
            trace!("✗ Dead end at MinigridIdx={}, Mask={}", self.id, used_mask);
        }
    }
}

impl<const N: usize, const K: usize> super::SudokuSolver<N, K> {
    pub fn generate_all_permutations(&self, masks: &Masks<N>) -> [Vec<PermutationNode<N, K>>; N] {
        debug!(
            "Starting parallel permutation generation for {} minigrid(s)",
            N
        );

        (0..N)
            .into_par_iter()
            .map(|id| {
                let mut mg = Minigrid::new(id, &self.board);
                let mut results = Vec::new();

                // Used mask tracks numbers already present in the minigrid
                let used_mask = masks.boxs[id];
                debug!(
                    "Generating permutations for Minigrid {} (initial_mask={})",
                    id, used_mask
                );
                mg.generate_permutations_dfs(&used_mask, masks, &mut results);
                debug!("Minigrid {} completed: {} solutions", id, results.len());

                results
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()

        // let id = 5; // Temporarily using single-threaded for easier debugging
        // let mut mg = Minigrid::new(id, &self.board);
        // let mut results = Vec::new();

        // // Used mask tracks numbers already present in the minigrid
        // let used_mask = masks.boxs[id];
        // debug!(
        //     "Generating permutations for Minigrid {} (InitialMask={})",
        //     id, used_mask
        // );
        // mg.generate_permutations_dfs(&used_mask, masks, &mut results);
        // debug!("Minigrid {} completed: {} solutions", id, results.len());
        // let mut all_permutations = [(); N].map(|_| Vec::new());
        // all_permutations[id] = results;
        // all_permutations
    }
}
