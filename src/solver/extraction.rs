use crate::types::{
    Board,
    graph::{Graph, Relation},
};
use log::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct Solution<const N: usize> {
    pub board: Board<N>,
    pub permutation_ids: [usize; N], // Which permutation used per minigrid
}

#[derive(Debug, Clone, PartialEq)]
pub enum PuzzleClass {
    Unsolvable,
    Unique,
    Ambiguous(usize),
}

/// Finds all complete, valid permutation assignments for the board.
///
/// Uses a backtracking search with the Minimum Remaining Values (MRV) heuristic
/// to efficiently explore the graph.
pub struct Search<'a, const K: usize, const N: usize> {
    graph: &'a Graph<K, N>,
    assignments: [Option<usize>; N],
    solutions: Vec<[usize; N]>,
    limit: Option<usize>,
}

impl<'a, const K: usize, const N: usize> Search<'a, K, N> {
    pub fn new(graph: &'a Graph<K, N>) -> Self {
        Self {
            graph,
            assignments: [None; N],
            solutions: Vec::new(),
            limit: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn all(mut self) -> Vec<[usize; N]> {
        self.backtrack(0);
        self.solutions
    }

    fn backtrack(&mut self, assigned_count: usize) {
        if let Some(limit) = self.limit
            && self.solutions.len() >= limit
        {
            return;
        }

        if assigned_count == N {
            let complete = core::array::from_fn(|i| self.assignments[i].unwrap());
            self.solutions.push(complete);
            return;
        }

        let Some(mg_id) = self.next_minigrid() else {
            return; // Dead end
        };

        for perm_id in 0..self.graph.permutation_count(mg_id) {
            if self.is_compatible(mg_id, perm_id) {
                self.assignments[mg_id] = Some(perm_id);
                self.backtrack(assigned_count + 1);
                self.assignments[mg_id] = None;
            }
        }
    }

    fn next_minigrid(&self) -> Option<usize> {
        let mut best_mg = None;
        let mut min_candidates = usize::MAX;

        for mg_id in 0..N {
            if self.assignments[mg_id].is_some() {
                continue;
            }

            let mut count = 0;
            for perm_id in 0..self.graph.permutation_count(mg_id) {
                if self.is_compatible(mg_id, perm_id) {
                    count += 1;
                }
            }

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

    fn is_compatible(&self, mg_id: usize, perm_id: usize) -> bool {
        let edges = self.graph.compatible_edges(mg_id, perm_id);

        for (other_mg, assigned) in self.assignments.iter().enumerate().take(N) {
            if let Some(other_perm) = *assigned
                && self.graph.relationship(mg_id, other_mg) != Relation::Not
                && !edges.contains(&(other_mg, other_perm))
            {
                return false;
            }
        }

        true
    }
}

/// Extract all valid solutions from a pruned graph.
///
/// Converts the configurations identified by the exact global support search
/// into fully reconstructed Sudoku boards.
pub struct Extractor<'a, const K: usize, const N: usize> {
    graph: &'a Graph<K, N>,
    limit: Option<usize>,
}

impl<'a, const K: usize, const N: usize> Extractor<'a, K, N> {
    pub fn new(graph: &'a Graph<K, N>) -> Self {
        Self { graph, limit: None }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn run(&self) -> Vec<Solution<N>> {
        let mut search = Search::new(self.graph);
        if let Some(limit) = self.limit {
            search = search.with_limit(limit);
        }
        let configurations = search.all();
        let mut solutions = Vec::with_capacity(configurations.len());

        for config in configurations {
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
