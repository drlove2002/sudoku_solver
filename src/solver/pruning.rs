use crate::solver::extraction::Search;
use crate::types::graph::Graph;
use log::{debug, info};

pub struct PruneResult<const N: usize> {
    pub removed_total: usize,
    pub configurations: Vec<[usize; N]>,
}

/// Prune permutations that do not participate in any globally consistent board.
///
/// This performs an exact global support search (finding all valid configurations)
/// and records every permutation that appears in at least one valid configuration.
/// It then rebuilds the graph using only those supported nodes.
pub struct Pruner<'a, const K: usize, const N: usize> {
    graph: &'a mut Graph<K, N>,
}

impl<'a, const K: usize, const N: usize> Pruner<'a, K, N> {
    pub fn new(graph: &'a mut Graph<K, N>) -> Self {
        Self { graph }
    }

    pub fn run(&mut self) -> PruneResult<N> {
        let original_counts: [usize; N] =
            std::array::from_fn(|mg_id| self.graph.permutation_count(mg_id));

        self.run_local();

        let total_after_local: usize = original_counts
            .iter()
            .zip(std::array::from_fn::<_, N, _>(|mg_id| self.graph.permutation_count(mg_id)))
            .map(|(a, b)| a - b)
            .sum();
        info!(
            "Local pruning removed {} permutation(s)",
            total_after_local
        );

        let current_total = self.graph.total_permutations();
        // Exact global support search is O(#configs × N × P).
        // For very large graphs the benefit (shrinking pair tables for Phase 5)
        // is outweighed by the cost. Skip and let the extractor search directly.
        const EXACT_PRUNING_THRESHOLD: usize = 200_000;
        if current_total > EXACT_PRUNING_THRESHOLD {
            info!(
                "Skipping exact pruning ({} perms > {} threshold) — extractor will search directly",
                current_total, EXACT_PRUNING_THRESHOLD
            );
            return PruneResult {
                removed_total: total_after_local,
                configurations: Vec::new(), // empty → extractor runs its own search
            };
        }

        // Run the exact global search to find all valid configurations
        let configurations = Search::new(self.graph).all();
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

        let remap: [Vec<Option<usize>>; N] = std::array::from_fn(|mg_id| {
            let mut mapping = vec![None; self.graph.permutation_count(mg_id)];
            for (new_idx, &old_idx) in keep[mg_id].iter().enumerate() {
                mapping[old_idx] = Some(new_idx);
            }
            mapping
        });

        self.graph.retain_permutations(&keep);

        let configurations = configurations
            .into_iter()
            .map(|config| {
                std::array::from_fn(|mg_id| {
                    remap[mg_id][config[mg_id]]
                        .expect("exact search configurations must survive support pruning")
                })
            })
            .collect();

        info!(
            "Pruning complete: removed {} permutation(s) across {} full configuration(s)",
            removed_total, solution_count
        );

        PruneResult {
            removed_total,
            configurations,
        }
    }

    pub fn run_local(&mut self) -> usize {
        let original_total = self.graph.total_permutations();
        let local_keep = self.local_support_keep();
        self.graph.retain_permutations(&local_keep);
        original_total - self.graph.total_permutations()
    }

    fn local_support_keep(&self) -> [Vec<usize>; N] {
        let mut alive: [Vec<bool>; N] =
            std::array::from_fn(|mg_id| vec![true; self.graph.permutation_count(mg_id)]);
        let mut changed = true;

        while changed {
            changed = false;

            for mg_id in 0..N {
                for perm_id in 0..self.graph.permutation_count(mg_id) {
                    if !alive[mg_id][perm_id] {
                        continue;
                    }

                    let mut supported = true;
                    for (other_mg, other_alive) in alive.iter().enumerate() {
                        if self.graph.relationship(mg_id, other_mg)
                            == crate::types::graph::Relation::Not
                        {
                            continue;
                        }

                        let compatible = self
                            .graph
                            .compatible_set(mg_id, perm_id, other_mg)
                            .expect("related minigrids must have compatibility data");

                        let has_live_neighbor = compatible
                            .iter_ones()
                            .any(|other_perm| other_alive[other_perm]);

                        if !has_live_neighbor {
                            supported = false;
                            break;
                        }
                    }

                    if !supported {
                        alive[mg_id][perm_id] = false;
                        changed = true;
                    }
                }
            }
        }

        std::array::from_fn(|mg_id| {
            alive[mg_id]
                .iter()
                .enumerate()
                .filter_map(|(perm_id, &is_alive)| is_alive.then_some(perm_id))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SudokuSolver,
        types::{Board, graph::Graph, masks::Masks},
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
        let permutations =
            crate::solver::permutations::generate_all_permutations(&solver.board, &masks);
        let mut graph = Graph::new(permutations);
        graph.create_edges();
        graph
    }

    #[test]
    fn test_exact_pruning_preserves_solutions_for_known_puzzle() {
        let puzzle =
            "...81.....2........1.9..7...7..25.934.2............5...975.....563.....4......68.";
        let mut graph = graph_from_puzzle(puzzle);

        let result = Pruner::new(&mut graph).run();

        assert!(
            result.removed_total > 0,
            "expected unsupported permutations to be pruned"
        );
        assert!(
            graph.total_permutations() > 0,
            "valid puzzle was over-pruned"
        );
        assert!(
            !result.configurations.is_empty(),
            "expected exact pruning to preserve at least one full configuration"
        );
    }
}
