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
use log::{debug, info};
use pruning::{PruneResult, Pruner};
use report::{PuzzleClass, SearchMode, SolveReport, SolveStats};
use std::time::Instant;

pub struct SudokuSolver<const N: usize, const K: usize> {
    pub board: Board<N>,
    pub search_mode: SearchMode,
    pub visualize: bool,
}

impl<const N: usize, const K: usize> SudokuSolver<N, K> {
    pub fn new(board: Board<N>) -> Self {
        SudokuSolver {
            board,
            search_mode: SearchMode::EnumerateAll,
            visualize: false,
        }
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

        info!("=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        let phase_start = Instant::now();
        let mut masks = Masks::<N>::default();
        masks.generate(&self.board);
        let mask_init_time_ns = phase_start.elapsed().as_nanos();
        info!("✓ Initial allowed masks pre-calculated (optimized)");

        info!("=== PHASE 1.5: CONSTRAINT PROPAGATION ===");
        let phase_start = Instant::now();
        let mut heuristic_board = self.board;
        let cells_filled =
            heuristics::propagate_constraints::<N, K>(&mut heuristic_board, &mut masks);
        let heuristic_time_ns = phase_start.elapsed().as_nanos();
        info!(
            "✓ Filled {} deterministic cells via constraint propagation (naked + hidden singles)",
            cells_filled
        );

        info!("=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        let phase_start = Instant::now();
        let permutations: [GeneratedMinigrid<N, K>; N] =
            permutations::generate_all_permutations(&heuristic_board, &masks);
        let permutation_time_ns = phase_start.elapsed().as_nanos();
        let permutation_counts = std::array::from_fn(|idx| permutations[idx].nodes.len());
        let total_invocations = count_dependent_pair_checks(&permutations);
        let perm_mem = permutation_memory::<N, K>(&permutations);

        // Print permutation counts and details
        for (idx, perms) in permutations.iter().enumerate() {
            info!("Minigrid {}: {} permutation(s)", idx, perms.nodes.len());
            for (p_idx, cells) in perms.payloads.iter().enumerate() {
                debug!("  M-{}-{}: {}", idx, p_idx, format_minigrid::<N, K>(cells));
            }
        }

        // Check for capped minigrids — bail gracefully
        let capped_count = permutations.iter().filter(|p| p.capped).count();
        if capped_count > 0 {
            let capped_ids: Vec<usize> = permutations
                .iter()
                .enumerate()
                .filter_map(|(i, p)| p.capped.then_some(i))
                .collect();
            info!(
                "{} minigrid(s) exceeded the {} permutation cap: {:?}",
                capped_count, 100_000, capped_ids
            );
            info!("Aborting — the minigrid-decomposition approach cannot solve this board size at present.");
            let total_time_ns = total_start.elapsed().as_nanos();
            return SolveReport {
                solutions: Vec::new(),
                stats: SolveStats {
                    permutation_counts,
                    total_invocations,
                    initial_vertex_count: 0,
                    initial_edge_count: 0,
                    pruned_vertex_count: 0,
                    pruned_edge_count: 0,
                    removed_vertices: 0,
                    solution_count: 0,
                    puzzle_classification: PuzzleClass::Unsolvable,
                    mask_init_time_ns,
                    heuristic_time_ns,
                    permutation_time_ns,
                    edge_build_time_ns: 0,
                    pruning_time_ns: 0,
                    extraction_time_ns: 0,
                    total_time_ns,
                    masks_memory_bytes: 0,
                    permutation_memory_bytes: 0,
                    graph_memory_bytes: 0,
                    post_prune_memory_bytes: 0,
                },
            };
        }

        info!("=== PHASE 3: GRAPH CONSTRUCTION ===");
        let phase_start = Instant::now();
        let mut graph = Graph::new(permutations);
        let initial_perms = graph.total_permutations();
        info!("Initial graph: {} permutation(s)", initial_perms);

        graph.create_edges();
        let edge_build_time_ns = phase_start.elapsed().as_nanos();
        let initial_edge_count = graph.total_edges();
        let graph_mem = graph.memory_usage();

        // Detailed graph memory breakdown
        let breakdown = graph.memory_detailed();
        info!(
            "   Nodes:              {:>6} ({:.1} MB)",
            breakdown.nodes_bytes,
            breakdown.nodes_bytes as f64 / 1_048_576.0
        );
        info!(
            "   Payloads:           {:>6} ({:.1} MB)",
            breakdown.payloads_bytes,
            breakdown.payloads_bytes as f64 / 1_048_576.0
        );
        info!(
            "   Row signatures:     {:>6}",
            breakdown.row_signatures_bytes
        );
        info!(
            "   Row perm→sig:       {:>6}",
            breakdown.row_perm_to_sig_bytes
        );
        info!(
            "   Row sig→perms:      {:>6} ({:.1} MB)",
            breakdown.row_sig_to_perms_bytes,
            breakdown.row_sig_to_perms_bytes as f64 / 1_048_576.0
        );
        info!(
            "   Col signatures:     {:>6}",
            breakdown.col_signatures_bytes
        );
        info!(
            "   Col perm→sig:       {:>6}",
            breakdown.col_perm_to_sig_bytes
        );
        info!(
            "   Col sig→perms:      {:>6} ({:.1} MB)",
            breakdown.col_sig_to_perms_bytes,
            breakdown.col_sig_to_perms_bytes as f64 / 1_048_576.0
        );
        info!(
            "   Pair L→R:           {:>6} ({:.1} MB)",
            breakdown.pair_tables_left_bytes,
            breakdown.pair_tables_left_bytes as f64 / 1_048_576.0
        );
        info!(
            "   Pair R→L:           {:>6} ({:.1} MB)",
            breakdown.pair_tables_right_bytes,
            breakdown.pair_tables_right_bytes as f64 / 1_048_576.0
        );
        info!("   Pair count:         {}", breakdown.pair_count);

        let sig_perms_total = breakdown.row_sig_to_perms_bytes + breakdown.col_sig_to_perms_bytes;
        let pair_total = breakdown.pair_tables_left_bytes + breakdown.pair_tables_right_bytes;
        info!(
            "   ── sig→perms (row+col): {:>6} ({:.1} MB)",
            sig_perms_total,
            sig_perms_total as f64 / 1_048_576.0
        );
        info!(
            "   ── pair tables (L+R): {:>6} ({:.1} MB)",
            pair_total,
            pair_total as f64 / 1_048_576.0
        );

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
        let PruneResult {
            removed_total: removed,
            configurations,
        } = pruner.run();
        let pruning_time_ns = phase_start.elapsed().as_nanos();
        let final_perms = graph.total_permutations();
        let pruned_edge_count = graph.total_edges();
        info!(
            "✓ Pruning complete: {} → {} permutation(s) ({} removed)",
            initial_perms, final_perms, removed
        );

        info!("=== PHASE 5: SOLUTION EXTRACTION ===");
        let phase_start = Instant::now();
        let extractor = Extractor::new(&graph).with_mode(self.search_mode);
        let solutions = extractor.run_with_configurations(configurations);
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

        let masks_mem = (std::mem::size_of::<Masks<N>>() + std::mem::size_of::<Board<N>>()) as u64;
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
                mask_init_time_ns,
                heuristic_time_ns,
                permutation_time_ns,
                edge_build_time_ns,
                pruning_time_ns,
                extraction_time_ns,
                total_time_ns,
                masks_memory_bytes: masks_mem,
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
        heuristics::propagate_constraints::<N, K>(&mut heuristic_board, &mut masks);

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
    let payloads_size = N; // [u8; N]
    let mut bytes = 0u64;
    for mg in perms {
        bytes += (mg.nodes.capacity() * node_size + mg.payloads.capacity() * payloads_size) as u64;
    }
    bytes
}

fn format_minigrid<const N: usize, const K: usize>(cells: &[u8; N]) -> String {
    let mut out = String::from("[");
    for (i, val) in cells.iter().enumerate() {
        if i > 0 {
            if i % K == 0 {
                out.push_str(" | ");
            } else {
                out.push(' ');
            }
        }
        out.push_str(&val.to_string());
    }
    out.push(']');
    out
}
