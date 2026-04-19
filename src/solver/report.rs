use crate::types::Board;

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
