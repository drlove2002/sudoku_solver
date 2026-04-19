use crate::solver::extraction::Search;
use crate::types::graph::Graph;
use log::{debug, info};

/// Prune permutations that do not participate in any globally consistent board.
///
/// This performs an exact global support search (finding all valid configurations)
/// and records every permutation that appears in at least one valid configuration.
/// It then rebuilds the graph using only those supported nodes.
pub struct Pruner<'a, const K: usize, const N: usize> {
    graph: &'a mut Graph<K, N>,
    limit: Option<usize>,
}

impl<'a, const K: usize, const N: usize> Pruner<'a, K, N> {
    pub fn new(graph: &'a mut Graph<K, N>) -> Self {
        Self { graph, limit: None }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn run(&mut self) -> usize {
        let original_counts: [usize; N] =
            std::array::from_fn(|mg_id| self.graph.permutation_count(mg_id));

        // Run the exact global search to find all valid configurations
        let mut search = Search::new(self.graph);
        if let Some(limit) = self.limit {
            search = search.with_limit(limit);
        }
        let configurations = search.all();
        let solution_count = configurations.len();

        // Mark permutations that are used in at least one solution
        let mut supported: [Vec<bool>; N] =
            std::array::from_fn(|mg_id| vec![false; original_counts[mg_id]]);

        for config in &configurations {
            for (mg_id, &perm_id) in config.iter().enumerate() {
                supported[mg_id][perm_id] = true;
            }
        }

        // Collect the IDs of supported permutations to keep
        let keep: [Vec<usize>; N] = std::array::from_fn(|mg_id| {
            supported[mg_id]
                .iter()
                .enumerate()
                .filter_map(
                    |(perm_id, &is_supported)| {
                        if is_supported { Some(perm_id) } else { None }
                    },
                )
                .collect()
        });

        let removed_total: usize = original_counts
            .iter()
            .zip(keep.iter())
            .map(|(original, keep)| original - keep.len())
            .sum();

        for mg_id in 0..N {
            debug!(
                "MG{} supported permutations: {} / {}",
                mg_id,
                keep[mg_id].len(),
                original_counts[mg_id]
            );
        }

        self.graph.retain_permutations(&keep);

        info!(
            "Pruning complete: removed {} permutation(s) across {} full configuration(s)",
            removed_total, solution_count
        );

        removed_total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SudokuSolver,
        types::{graph::Graph, masks::Masks, Board},
        utils::dataset::parse_puzzle_string,
    };

    const N: usize = 9;
    const K: usize = 3;

    fn graph_from_puzzle(puzzle: &str) -> Graph<K, N> {
        let cells = parse_puzzle_string(puzzle).expect("valid puzzle");
        let mut board_cells = [[0u8; N]; N];
        for (idx, value) in cells.into_iter().enumerate() {
            board_cells[idx / N][idx % N] = value;
        }

        let board = Board::<N>::new(board_cells);
        let solver = SudokuSolver::<N, K>::new(board);
        let mut masks = Masks::<N>::default();
        masks.generate(&solver.board);
        let permutations = crate::solver::permutations::generate_all_permutations(&solver.board, &masks);
        let mut graph = Graph::new(permutations);
        graph.create_edges();
        graph
    }

    #[test]
    fn test_exact_pruning_preserves_solutions_for_known_puzzle() {
        let puzzle =
            "...81.....2........1.9..7...7..25.934.2............5...975.....563.....4......68.";
        let mut graph = graph_from_puzzle(puzzle);

        let removed = Pruner::new(&mut graph).run();

        assert!(
            removed > 0,
            "expected unsupported permutations to be pruned"
        );
        assert!(
            graph.total_permutations() > 0,
            "valid puzzle was over-pruned"
        );
    }
}
