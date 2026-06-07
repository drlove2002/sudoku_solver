use super::report::{SearchMode, Solution};
use crate::types::{
    Board,
    bitstring::DynamicBitSet,
    graph::{Graph, Relation},
};
use log::{debug, info, warn};

/// Finds all complete, valid permutation assignments for the board.
///
/// Uses a backtracking search with the Minimum Remaining Values (MRV) heuristic
/// to efficiently explore the graph.
pub struct Search<'a, const K: usize, const N: usize> {
    graph: &'a Graph<K, N>,
    assignments: [Option<usize>; N],
    domains: [DynamicBitSet; N],
    solutions: Vec<[usize; N]>,
    trail: Vec<TrailEntry>,
    touched_at: [usize; N],
    decision_level: usize,
    mode: SearchMode,
}

#[derive(Debug, Clone)]
struct TrailEntry {
    mg_id: usize,
    domain: DynamicBitSet,
    assignment: Option<usize>,
}

impl<'a, const K: usize, const N: usize> Search<'a, K, N> {
    pub fn new(graph: &'a Graph<K, N>) -> Self {
        Self {
            graph,
            assignments: [None; N],
            domains: std::array::from_fn(|mg_id| {
                DynamicBitSet::full(graph.permutation_count(mg_id))
            }),
            solutions: Vec::new(),
            trail: Vec::new(),
            touched_at: [usize::MAX; N],
            decision_level: 0,
            mode: SearchMode::EnumerateAll,
        }
    }

    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn all(mut self) -> Vec<[usize; N]> {
        if (0..N).any(|mg_id| self.graph.permutation_count(mg_id) == 0) {
            return self.solutions;
        }

        self.backtrack();
        self.solutions
    }

    fn backtrack(&mut self) {
        if let Some(limit) = self.mode.solution_cap()
            && self.solutions.len() >= limit
        {
            return;
        }

        if self.assignments.iter().all(Option::is_some) {
            let complete = core::array::from_fn(|i| {
                self.assignments[i].expect("complete assignment must include every minigrid")
            });
            self.solutions.push(complete);
            return;
        }

        let Some(mg_id) = self.next_minigrid() else {
            return; // Dead end
        };

        let candidates: Vec<usize> = self.domains[mg_id].iter_ones().collect();
        for perm_id in candidates {
            let marker = self.trail.len();
            self.decision_level += 1;

            if self.assign_and_propagate(mg_id, perm_id) {
                self.backtrack();
            }

            self.undo_to(marker);
            self.decision_level -= 1;
        }
    }

    fn next_minigrid(&self) -> Option<usize> {
        let mut best_mg = None;
        let mut min_candidates = usize::MAX;

        for mg_id in 0..N {
            if self.assignments[mg_id].is_some() {
                continue;
            }

            let count = self.domains[mg_id].count_ones();

            if count < min_candidates {
                min_candidates = count;
                best_mg = Some(mg_id);
            }

            if min_candidates == 0 {
                break; // Fast fail
            }
        }

        best_mg
    }

    fn assign_and_propagate(&mut self, mg_id: usize, perm_id: usize) -> bool {
        let mut queue = vec![(mg_id, perm_id)];
        self.save_domain(mg_id);
        self.assignments[mg_id] = Some(perm_id);
        self.domains[mg_id] =
            DynamicBitSet::singleton(self.graph.permutation_count(mg_id), perm_id);

        while let Some((current_mg, current_perm)) = queue.pop() {
            for other_mg in 0..N {
                let relation = self.graph.relationship(current_mg, other_mg);
                if relation == Relation::Not {
                    continue;
                }

                let compatible = self
                    .graph
                    .compatible_set(current_mg, current_perm, other_mg)
                    .expect("related minigrids must have compatibility data");

                if let Some(other_perm) = self.assignments[other_mg] {
                    if !compatible.contains(other_perm) {
                        return false;
                    }
                    continue;
                }

                self.save_domain(other_mg);
                let changed = self.domains[other_mg].intersect_with_fixed(compatible);
                if self.domains[other_mg].is_empty() {
                    return false;
                }

                if changed && self.domains[other_mg].count_ones() == 1 {
                    let forced_perm = self.domains[other_mg]
                        .iter_ones()
                        .next()
                        .expect("singleton domain must contain one permutation");
                    self.assignments[other_mg] = Some(forced_perm);
                    queue.push((other_mg, forced_perm));
                }
            }
        }

        true
    }

    fn save_domain(&mut self, mg_id: usize) {
        if self.touched_at[mg_id] == self.decision_level {
            return;
        }

        self.touched_at[mg_id] = self.decision_level;
        self.trail.push(TrailEntry {
            mg_id,
            domain: self.domains[mg_id].clone(),
            assignment: self.assignments[mg_id],
        });
    }

    fn undo_to(&mut self, marker: usize) {
        while self.trail.len() > marker {
            let entry = self.trail.pop().expect("trail marker must be valid");
            self.domains[entry.mg_id] = entry.domain;
            self.assignments[entry.mg_id] = entry.assignment;
            self.touched_at[entry.mg_id] = usize::MAX;
        }
    }
}

/// Extract all valid solutions from a pruned graph.
///
/// Converts the configurations identified by the exact global support search
/// into fully reconstructed Sudoku boards.
pub struct Extractor<'a, const K: usize, const N: usize> {
    graph: &'a Graph<K, N>,
    mode: SearchMode,
}

impl<'a, const K: usize, const N: usize> Extractor<'a, K, N> {
    pub fn new(graph: &'a Graph<K, N>) -> Self {
        Self {
            graph,
            mode: SearchMode::EnumerateAll,
        }
    }

    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn run(&self) -> Vec<Solution<N>> {
        let search = Search::new(self.graph).with_mode(self.mode);
        let configurations = search.all();
        self.run_with_configurations(configurations)
    }

    pub fn run_with_configurations(&self, configurations: Vec<[usize; N]>) -> Vec<Solution<N>> {
        let configs = if configurations.is_empty() {
            // Pruning skipped exact support search — run it now
            let search = Search::new(self.graph).with_mode(self.mode);
            search.all()
        } else {
            configurations
        };

        let mut solutions = Vec::with_capacity(configs.len());

        for config in configs {
            let board = self.reconstruct(&config);

            if !board.is_valid() {
                warn!("Reconstructed invalid board from config: {:?}", config);
                continue;
            }

            debug!("Found valid solution with config: {:?}", config);
            solutions.push(Solution {
                board,
                permutation_ids: config,
            });
        }

        match solutions.len() {
            0 => warn!("No solutions found - puzzle is unsolvable"),
            1 => info!("Found unique solution"),
            n => info!("Found {} solutions - puzzle is ambiguous", n),
        }

        solutions
    }

    fn reconstruct(&self, config: &[usize; N]) -> Board<N> {
        let mut cells = [[0u8; N]; N];

        for (mg_id, &perm_id) in config.iter().enumerate() {
            let perm_cells = self.graph.permutation_cells(mg_id, perm_id);

            let mg_row = mg_id / K;
            let mg_col = mg_id % K;
            let base_row = mg_row * K;
            let base_col = mg_col * K;

            for (i, &cell) in perm_cells.iter().enumerate().take(N) {
                let local_row = i / K;
                let local_col = i % K;
                let board_row = base_row + local_row;
                let board_col = base_col + local_col;

                cells[board_row][board_col] = cell;
            }
        }

        Board::new(cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::graph::{GeneratedMinigrid, PermId, PermutationNode};

    fn generated_minigrid<const N: usize, const K: usize>(
        payloads: Vec<[u8; N]>,
    ) -> GeneratedMinigrid<N, K> {
        let nodes = payloads
            .iter()
            .enumerate()
            .map(|(idx, cells)| {
                PermutationNode::from_minigrid(
                    cells,
                    PermId::try_from(idx).expect("payload index must fit into u32"),
                )
            })
            .collect();
        GeneratedMinigrid { nodes, payloads }
    }

    #[test]
    fn search_trail_restores_domains_across_branches() {
        const K: usize = 2;
        const N: usize = 4;

        let mut graph = Graph::new([
            generated_minigrid::<N, K>(vec![[1, 2, 3, 4], [1, 3, 2, 4]]),
            generated_minigrid::<N, K>(vec![[3, 4, 1, 2], [2, 4, 1, 3]]),
            generated_minigrid::<N, K>(vec![[2, 1, 4, 3], [3, 1, 4, 2]]),
            generated_minigrid::<N, K>(vec![[4, 3, 2, 1], [4, 2, 3, 1]]),
        ]);
        graph.create_edges();

        let configs = Search::new(&graph)
            .with_mode(SearchMode::EnumerateAll)
            .all();

        assert_eq!(configs, vec![[0, 0, 0, 0], [1, 1, 1, 1]]);
    }
}
