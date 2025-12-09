pub mod permutations;
use crate::types::{Board, Permutations, masks::Masks};
use log::{debug, info};

pub struct SudokuSolver<const N: usize> {
    pub board: Board<N>,
}

impl<const N: usize> SudokuSolver<N> {
    pub fn new(board: Board<N>) -> Self {
        SudokuSolver { board }
    }

    pub fn solve(&self) -> [Permutations<N>; N] {
        info!("=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        let mut masks = Masks::<N>::default();
        masks.generate(&self.board);
        info!("✓ Initial allowed masks pre-calculated (optimized)");

        info!("=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        let permutations = self.generate_all_permutations(&masks);

        // Print permutation counts and details
        for (idx, perms) in permutations.iter().enumerate() {
            debug!("Minigrid {}: {} permutation(s)", idx, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                debug!("  M-{}-{}: {}", perm.id, p_idx, perm);
            }
        }
        permutations
    }
}
