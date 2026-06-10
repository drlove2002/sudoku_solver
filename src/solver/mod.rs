pub mod extraction;
pub mod heuristics;
pub mod permutations;
pub mod pruning;
pub mod report;

use crate::types::{
    Board,
    graph::{GeneratedMinigrid, Graph, PermutationNode},
    masks::Masks,
};
use extraction::Extractor;
use log::info;
use pruning::{PruneResult, Pruner};
use report::{PhaseProgress, PuzzleClass, SearchMode, SolveReport, SolveStats};
use std::time::Instant;

pub struct SudokuSolver<const N: usize, const K: usize> {
    pub board: Board<N>,
    pub search_mode: SearchMode,
    pub visualize: bool,
    pub use_heuristics: bool,
    pub breadcrumb_path: Option<String>,
}

impl<const N: usize, const K: usize> SudokuSolver<N, K> {
    pub fn new(board: Board<N>) -> Self {
        SudokuSolver {
            board,
            search_mode: SearchMode::EnumerateAll,
            visualize: false,
            use_heuristics: true,
            breadcrumb_path: None,
        }
    }

    pub fn with_breadcrumb(mut self, path: &str) -> Self {
        self.breadcrumb_path = Some(path.to_string());
        self
    }

    pub fn without_heuristics(mut self) -> Self {
        self.use_heuristics = false;
        self
    }

    pub fn with_heuristics(mut self, enabled: bool) -> Self {
        self.use_heuristics = enabled;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.search_mode = SearchMode::EnumerateUpTo(limit);
        self
    }

    pub fn with_search_mode(mut self, mode: SearchMode) -> Self {
        self.search_mode = mode;
        self
    }

    pub fn with_visualize(mut self, visualize: bool) -> Self {
        self.visualize = visualize;
        self
    }

    pub fn solve(&self) -> Vec<Board<N>> {
        self.solve_with_stats()
            .solutions
            .into_iter()
            .map(|s| s.board)
            .collect()
    }

    pub fn solve_one(&self) -> Option<Board<N>> {
        self.solve_bounded(SearchMode::First)
            .into_iter()
            .next()
            .map(|solution| solution.board)
    }

    pub fn classify_up_to_two(&self) -> PuzzleClass {
        match self.solve_bounded(SearchMode::Classify).len() {
            0 => PuzzleClass::Unsolvable,
            1 => PuzzleClass::Unique,
            _ => PuzzleClass::Ambiguous(2),
        }
    }

    pub fn solve_with_stats(&self) -> SolveReport<N> {
        let total_start = Instant::now();

        // Phase 1: Mask init
        let phase_start = Instant::now();
        let mut masks = Masks::<N>::default();
        masks.generate(&self.board);
        let mask_init_time_ns = phase_start.elapsed().as_nanos();

        // Phase 2: Constraint propagation
        let phase_start = Instant::now();
        let mut heuristic_board = self.board;
        let cells_filled = if self.use_heuristics {
            heuristics::propagate_constraints::<N, K>(&mut heuristic_board, &mut masks)
        } else {
            0
        };
        let heuristic_time_ns = phase_start.elapsed().as_nanos();

        // Breadcrumb: entering Phase 3
        write_breadcrumb(&self.breadcrumb_path, "permutations");

        // Phase 3: Permutation generation
        let phase_start = Instant::now();
        let permutations: [GeneratedMinigrid<N, K>; N] =
            permutations::generate_all_permutations(&heuristic_board, &masks);
        let permutation_time_ns = phase_start.elapsed().as_nanos();
        let permutation_counts = std::array::from_fn(|idx| permutations[idx].nodes.len());
        let total_invocations = count_dependent_pair_checks(&permutations);
        let perm_mem = permutation_memory::<N, K>(&permutations);

        // Breadcrumb: entering Phase 4
        write_breadcrumb(&self.breadcrumb_path, "graph");

        // Phase 4: Graph construction
        let phase_start = Instant::now();
        let mut graph = Graph::new(permutations);
        let initial_perms = graph.total_permutations();
        graph.create_edges();
        let edge_build_time_ns = phase_start.elapsed().as_nanos();
        let initial_edge_count = graph.total_edges();
        let graph_mem = graph.memory_usage();

        if self.visualize {
            info!("Exporting graph JSON for visualization...");
            std::fs::create_dir_all("results").unwrap_or_default();
            graph.export_to_json("results/graph.json");
        }

        // Breadcrumb: entering Phase 5
        write_breadcrumb(&self.breadcrumb_path, "pruning");

        // Phase 5: Pruning
        let phase_start = Instant::now();
        let mut pruner = Pruner::new(&mut graph);
        let PruneResult {
            removed_total: removed,
            configurations,
        } = pruner.run();
        let pruning_time_ns = phase_start.elapsed().as_nanos();
        let final_perms = graph.total_permutations();
        let pruned_edge_count = graph.total_edges();

        // Breadcrumb: entering Phase 6
        write_breadcrumb(&self.breadcrumb_path, "extraction");

        // Phase 6: Extraction
        let phase_start = Instant::now();
        let extractor = Extractor::new(&graph).with_mode(self.search_mode);
        let solutions = extractor.run_with_configurations(configurations);
        let extraction_time_ns = phase_start.elapsed().as_nanos();

        let classification = match solutions.len() {
            0 => PuzzleClass::Unsolvable,
            1 => PuzzleClass::Unique,
            n => PuzzleClass::Ambiguous(n),
        };

        let total_time_ns = total_start.elapsed().as_nanos();

        let masks_mem = (std::mem::size_of::<Masks<N>>() + std::mem::size_of::<Board<N>>()) as u64;
        let heuristic_mem = if self.use_heuristics { masks_mem } else { 0 };
        let post_prune_mem = graph.memory_usage();

        SolveReport {
            solutions,
            stats: SolveStats {
                permutation_counts,
                total_invocations,
                initial_vertex_count: initial_perms,
                initial_edge_count,
                pruned_vertex_count: final_perms,
                pruned_edge_count,
                removed_vertices: removed,
                solution_count: match classification {
                    PuzzleClass::Unsolvable => 0,
                    PuzzleClass::Unique => 1,
                    PuzzleClass::Ambiguous(n) => n,
                },
                puzzle_classification: classification,
                phase_progress: PhaseProgress::Complete,
                heuristic_used: self.use_heuristics,
                heuristic_cells_filled: cells_filled,
                mask_init_time_ns,
                heuristic_time_ns,
                permutation_time_ns,
                edge_build_time_ns,
                pruning_time_ns,
                extraction_time_ns,
                total_time_ns,
                masks_memory_bytes: masks_mem,
                heuristic_memory_bytes: heuristic_mem,
                permutation_memory_bytes: perm_mem,
                graph_memory_bytes: graph_mem,
                post_prune_memory_bytes: post_prune_mem,
            },
        }
    }

    fn solve_bounded(&self, mode: SearchMode) -> Vec<report::Solution<N>> {
        let mut masks = Masks::<N>::default();
        masks.generate(&self.board);

        let mut heuristic_board = self.board;
        if self.use_heuristics {
            heuristics::propagate_constraints::<N, K>(&mut heuristic_board, &mut masks);
        }

        let permutations: [GeneratedMinigrid<N, K>; N] =
            permutations::generate_all_permutations(&heuristic_board, &masks);
        let mut graph = Graph::new(permutations);
        graph.create_edges();

        Pruner::new(&mut graph).run_local();
        Extractor::new(&graph).with_mode(mode).run()
    }
}

fn count_dependent_pair_checks<const N: usize, const K: usize>(
    perms: &[GeneratedMinigrid<N, K>; N],
) -> usize {
    let mut total = 0;
    for i in 0..N {
        for j in (i + 1)..N {
            if Graph::<K, N>::relationship_between(i, j) != crate::types::graph::Relation::Not {
                total += perms[i].nodes.len() * perms[j].nodes.len();
            }
        }
    }
    total
}

fn permutation_memory<const N: usize, const K: usize>(perms: &[GeneratedMinigrid<N, K>; N]) -> u64 {
    let node_size = std::mem::size_of::<PermutationNode<N, K>>();
    let payloads_size = N;
    let mut bytes = 0u64;
    for mg in perms {
        bytes += (mg.nodes.capacity() * node_size + mg.payloads.capacity() * payloads_size) as u64;
    }
    bytes
}

fn write_breadcrumb(path: &Option<String>, phase: &str) {
    if let Some(p) = path {
        std::fs::write(p, phase).ok();
    }
}
