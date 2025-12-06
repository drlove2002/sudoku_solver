pub mod masks;
pub mod permutations;

use crate::types::{Board, Minigrid};
use log::{debug, info};
use masks::MaskGenerator;
use permutations::PermutationGenerator;

pub struct SudokuSolver<const N: usize> {
    pub board: Board<N>,
}

impl<const N: usize> SudokuSolver<N> {
    const K: usize = Board::<N>::K;

    pub fn new(board: Board<N>) -> Self {
        SudokuSolver { board }
    }

    pub fn solve(&mut self) -> Vec<Vec<Minigrid<N>>> {
        info!("=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        let masks = self.generate_masks();
        info!("✓ Initial allowed masks pre-calculated (optimized)");

        info!("=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        let permutations = self.generate_all_permutations(&masks);

        // Print permutation counts and details
        for (idx, perms) in permutations.iter().enumerate() {
            debug!("Minigrid {}: {} permutation(s)", idx, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                debug!("  P{}: {}", p_idx, perm);
            }
        }
        permutations
    }
}
