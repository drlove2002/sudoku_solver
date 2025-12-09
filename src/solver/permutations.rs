#![allow(dead_code)]
use crate::{
    helper::BitMask,
    types::{Minigrid, Permutations, masks::Masks},
};
use log::{debug, trace};
use rayon::prelude::*;

impl<const N: usize> Minigrid<N> {
    // Select the empty cell with the fewest candidates (MRV heuristic)
    // Returns Some(index) of the best cell, or None if no empty cells are found
    // MRV: Minimum Remaining Values
    #[inline(always)]
    fn find_best_cell(&self, used_mask: u32, masks: &Masks<N>) -> Option<usize> {
        let start_row = (self.id / Self::K) * Self::K;
        let start_col = (self.id % Self::K) * Self::K;

        let mut best_idx = None;
        let mut min_candidates = N + 1;

        let mut check_mask = self.empty_mask;
        while check_mask != 0 {
            let idx = check_mask.trailing_zeros() as usize;
            check_mask &= check_mask - 1; // Brian Kernighan's trick: remove lowest set bit
            if self.cells[idx] != 0 {
                continue;
            }

            let global_row = start_row + (idx / Self::K);
            let global_col = start_col + (idx % Self::K);
            let forbidden = masks.rows[global_row] | masks.cols[global_col] | used_mask;
            if forbidden == BitMask::<N>::all_set() {
                trace!(
                    "  Cell[{}] impossible at ({},{})",
                    idx, global_row, global_col
                );
                return None;
            }

            // Count remaining candidates for this cell
            let num_candidates = BitMask::<N>::candidates_count(forbidden);
            if num_candidates < min_candidates {
                // Less candidates found, update best choice
                min_candidates = num_candidates;
                best_idx = Some(idx);
                trace!(
                    "  Cell[{}] has {} candidates (MRV) mask {:032b}",
                    idx, num_candidates, forbidden
                );
                if num_candidates == 1 {
                    break;
                }
            }
        }

        debug!(
            "find_best_cell(mg={}, mask={:032b}): {:?}",
            self.id, used_mask, best_idx
        );
        best_idx
    }

    fn generate_permutations_dfs(
        &mut self,
        used_mask: u32,
        masks: &Masks<N>,
        results: &mut Vec<Minigrid<N>>,
    ) {
        if let Some(current_idx) = self.find_best_cell(used_mask, masks) {
            let start_row = (self.id / Self::K) * Self::K;
            let global_row = start_row + (current_idx / Self::K);
            let global_col = (self.id % Self::K) * Self::K + (current_idx % Self::K);
            let forbidden = masks.rows[global_row] | masks.cols[global_col] | used_mask;

            trace!(
                "DFS depth: mg={}, pos({},{}), mask={:032b}",
                self.id, global_row, global_col, used_mask
            );

            for num in 1..=N as u8 {
                let bit = BitMask::<N>::get(num);

                // Check if num can be placed
                if (forbidden & bit) == 0 {
                    trace!("  Try num={}", num);
                    self.cells[current_idx] = num;
                    self.generate_permutations_dfs(used_mask | bit, masks, results);
                    self.cells[current_idx] = 0; // Backtrack
                }
            }
        } else if BitMask::<N>::is_all_set(used_mask) {
            trace!("✓ Solution found for mg={}", self.id);
            results.push(*self);
        } else {
            trace!("✗ Dead end at mg={}, mask={:032b}", self.id, used_mask);
        }
    }
}

impl<const N: usize> super::SudokuSolver<N> {
    pub fn generate_all_permutations(&self, masks: &Masks<N>) -> [Permutations<N>; N] {
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
                    "Generating permutations for Minigrid {} (initial_mask={:032b})",
                    id, used_mask
                );
                mg.generate_permutations_dfs(used_mask, masks, &mut results);
                debug!("Minigrid {} completed: {} solutions", id, results.len());

                results
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()

        // let id = 2; // Temporarily using single-threaded for easier debugging
        // let mut mg = Minigrid::new(id, &self.board);
        // let mut results = Vec::new();

        // // Used mask tracks numbers already present in the minigrid
        // let used_mask = masks.boxs[id];
        // debug!(
        //     "Generating permutations for Minigrid {} (initial_mask={:032b})",
        //     id, used_mask
        // );
        // mg.generate_permutations_dfs(used_mask, masks, &mut results);
        // debug!("Minigrid {} completed: {} solutions", id, results.len());
        // let mut all_permutations: [Permutations<N>; N] = [(); N].map(|_| Vec::new());
        // all_permutations[id] = results;
        // all_permutations
    }
}
