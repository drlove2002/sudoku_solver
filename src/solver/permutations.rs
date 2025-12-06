use crate::types::{Minigrid, Permutations};
use rayon::prelude::*;

pub trait PermutationGenerator<const N: usize> {
    fn generate_all_permutations(&self, conflict_masks: &[Vec<u32>]) -> Vec<Permutations<N>>;
    fn generate_permutations_dfs(
        mg: Minigrid<N>,
        empty_indices: &[(usize, usize)],
        idx: usize,
        used_in_box: u32,
        conflict_masks: &[Vec<u32>],
        results: &mut Permutations<N>,
    );
}

impl<const N: usize> PermutationGenerator<N> for super::SudokuSolver<N> {
    fn generate_all_permutations(&self, conflict_masks: &[Vec<u32>]) -> Vec<Permutations<N>> {
        (0..N)
            .into_par_iter()
            .map(|id| {
                let mg = Minigrid::new(id, &self.board);
                let mut results = Vec::new();

                // Prepare for DFS
                let mut empty_indices = Vec::new();
                let mut used_in_box = 0u32;

                for r in 0..Self::K {
                    for c in 0..Self::K {
                        let val = mg.cells[r * Self::K + c];
                        if val == 0 {
                            empty_indices.push((r, c));
                        } else {
                            used_in_box |= 1 << val;
                        }
                    }
                }

                Self::generate_permutations_dfs(
                    mg,
                    &empty_indices,
                    0,
                    used_in_box,
                    conflict_masks,
                    &mut results,
                );
                results
            })
            .collect::<Vec<_>>()
    }

    fn generate_permutations_dfs(
        mut mg: Minigrid<N>,
        empty_indices: &[(usize, usize)],
        idx: usize,
        used_in_box: u32,
        conflict_masks: &[Vec<u32>],
        results: &mut Permutations<N>,
    ) {
        if idx == empty_indices.len() {
            results.push(mg);
            return;
        }

        let (r, c) = empty_indices[idx];

        // Calculate global coordinates
        let start_row = (mg.id / Self::K) * Self::K;
        let start_col = (mg.id % Self::K) * Self::K;
        let global_r = start_row + r;
        let global_c = start_col + c;

        let allowed_mask = conflict_masks[global_r][global_c];

        // Candidates are allowed by board constraints AND not used in this box
        let mut candidates = allowed_mask & !used_in_box;

        while candidates != 0 {
            let digit_bit = candidates & (!candidates + 1); // Lowest set bit
            candidates ^= digit_bit; // Remove it
            let digit = digit_bit.trailing_zeros() as u8;

            mg.cells[r * Self::K + c] = digit;
            Self::generate_permutations_dfs(
                mg,
                empty_indices,
                idx + 1,
                used_in_box | digit_bit,
                conflict_masks,
                results,
            );
        }
    }
}
