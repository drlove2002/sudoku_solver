pub mod extraction;
pub mod heuristics;
pub mod permutations;
pub mod pruning;

use crate::types::{
    Board,
    graph::{Graph, PermutationNode},
    masks::Masks,
};
use extraction::{Extractor, PuzzleClass, Solution};
use log::{debug, info};
use pruning::Pruner;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct SolveStats<const N: usize> {
    pub permutation_counts: [usize; N],
    pub total_invocations: usize,
    pub initial_vertex_count: usize,
    pub initial_edge_count: usize,
    pub pruned_vertex_count: usize,
    pub pruned_edge_count: usize,
    pub removed_vertices: usize,
    pub solution_count: usize,
    pub puzzle_classification: PuzzleClass,
    pub mask_init_time_ns: u128,
    pub heuristic_time_ns: u128,
    pub permutation_time_ns: u128,
    pub edge_build_time_ns: u128,
    pub pruning_time_ns: u128,
    pub extraction_time_ns: u128,
    pub total_time_ns: u128,
}

#[derive(Debug, Clone)]
pub struct SolveReport<const N: usize> {
    pub solutions: Vec<Solution<N>>,
    pub stats: SolveStats<N>,
}

pub struct SudokuSolver<const N: usize, const K: usize> {
    pub board: Board<N>,
    pub limit: Option<usize>,
    pub visualize: bool,
}

impl<const N: usize, const K: usize> SudokuSolver<N, K> {
    pub fn new(board: Board<N>) -> Self {
        SudokuSolver {
            board,
            limit: None,
            visualize: false,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
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

    pub fn solve_with_stats(&self) -> SolveReport<N> {
        let total_start = Instant::now();

        info!("=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        let phase_start = Instant::now();
        let mut masks = Masks::<N>::default();
        masks.generate(&self.board);
        let mask_init_time_ns = phase_start.elapsed().as_nanos();
        info!("✓ Initial allowed masks pre-calculated (optimized)");

        info!("=== PHASE 1.5: HIDDEN SINGLE DEDUCTION ===");
        let phase_start = Instant::now();
        let mut heuristic_board = self.board;
        let cells_filled = heuristics::apply_hidden_singles::<N, K>(&mut heuristic_board, &mut masks);
        let heuristic_time_ns = phase_start.elapsed().as_nanos();
        info!("✓ Filled {} deterministic cells via Hidden Singles", cells_filled);

        info!("=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        let phase_start = Instant::now();
        let permutations: [Vec<PermutationNode<N, K>>; N] = self.generate_all_permutations(&heuristic_board, &masks);
        let permutation_time_ns = phase_start.elapsed().as_nanos();
        let permutation_counts = std::array::from_fn(|idx| permutations[idx].len());
        let total_invocations = count_dependent_pair_checks(&permutations);

        // Print permutation counts and details
        for (idx, perms) in permutations.iter().enumerate() {
            info!("Minigrid {}: {} permutation(s)", idx, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                debug!("  M-{}-{}: {}", idx, p_idx, perm);
            }
        }

        info!("=== PHASE 3: GRAPH CONSTRUCTION ===");
        let phase_start = Instant::now();
        let mut graph = Graph::new(permutations);
        let initial_perms = graph.total_permutations();
        info!("Initial graph: {} permutation(s)", initial_perms);

        graph.create_edges();
        let edge_build_time_ns = phase_start.elapsed().as_nanos();
        let initial_edge_count = graph.total_edges();

        // Debug: print degrees before pruning
        for mg_id in 0..N {
            for (perm_id, degree) in graph.permutation_degrees(mg_id) {
                debug!("MG{}-P{}: degree = {}", mg_id, perm_id, degree);
            }
        }

        info!("✓ Compatibility edges built");

        if self.visualize {
            info!("Exporting graph JSON for visualization...");
            std::fs::create_dir_all("results").unwrap_or_default();
            graph.export_to_json("results/graph.json");
        }

        info!("=== PHASE 4: GRAPH PRUNING ===");
        let phase_start = Instant::now();
        let mut pruner = Pruner::new(&mut graph);
        if let Some(limit) = self.limit {
            pruner = pruner.with_limit(limit);
        }
        let removed = pruner.run();
        let pruning_time_ns = phase_start.elapsed().as_nanos();
        let final_perms = graph.total_permutations();
        let pruned_edge_count = graph.total_edges();
        info!(
            "✓ Pruning complete: {} → {} permutation(s) ({} removed)",
            initial_perms, final_perms, removed
        );

        info!("=== PHASE 5: SOLUTION EXTRACTION ===");
        let phase_start = Instant::now();
        let mut extractor = Extractor::new(&graph);
        if let Some(limit) = self.limit {
            extractor = extractor.with_limit(limit);
        }
        let solutions = extractor.run();
        let extraction_time_ns = phase_start.elapsed().as_nanos();

        // Classify puzzle
        let classification = match solutions.len() {
            0 => PuzzleClass::Unsolvable,
            1 => PuzzleClass::Unique,
            n => PuzzleClass::Ambiguous(n),
        };

        info!("Puzzle classification: {:?}", classification);

        // Display solutions
        for (idx, solution) in solutions.iter().enumerate() {
            info!(
                "Solution {} (Permutations: {:?}):",
                idx + 1,
                solution.permutation_ids
            );
            info!("\n{}", solution.board);
        }

        let total_time_ns = total_start.elapsed().as_nanos();

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
                mask_init_time_ns,
                heuristic_time_ns,
                permutation_time_ns,
                edge_build_time_ns,
                pruning_time_ns,
                extraction_time_ns,
                total_time_ns,
            },
        }
    }
}

fn count_dependent_pair_checks<const N: usize, const K: usize>(
    perms: &[Vec<PermutationNode<N, K>>; N],
) -> usize {
    let graph = Graph::<K, N>::new(std::array::from_fn(|idx| perms[idx].clone()));

    let mut total = 0;
    for i in 0..N {
        for j in (i + 1)..N {
            if graph.relationship(i, j) != crate::types::graph::Relation::Not {
                total += perms[i].len() * perms[j].len();
            }
        }
    }

    total
}
