// pub mod graph;
pub mod permutations;
use crate::types::{
    Board,
    graph::{Graph, PermutationNode},
    masks::Masks,
};
use log::{debug, info};

pub struct SudokuSolver<const N: usize, const K: usize> {
    pub board: Board<N>,
}

impl<const N: usize, const K: usize> SudokuSolver<N, K> {
    pub fn new(board: Board<N>) -> Self {
        SudokuSolver { board }
    }

    pub fn solve(&self) {
        info!("=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        let mut masks = Masks::<N>::default();
        masks.generate(&self.board);
        info!("✓ Initial allowed masks pre-calculated (optimized)");

        info!("=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        let permutations: [Vec<PermutationNode<N, K>>; N] = self.generate_all_permutations(&masks);

        // Print permutation counts and details
        for (idx, perms) in permutations.iter().enumerate() {
            info!("Minigrid {}: {} permutation(s)", idx, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                debug!("  M-{}-{}: {}", idx, p_idx, perm);
            }
        }

        let _graph = Graph::new(permutations);
    }
}
