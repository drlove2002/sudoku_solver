use crate::types::Board;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    First,
    Classify,
    EnumerateAll,
    EnumerateUpTo(usize),
}

impl SearchMode {
    pub fn solution_cap(self) -> Option<usize> {
        match self {
            SearchMode::First => Some(1),
            SearchMode::Classify => Some(2),
            SearchMode::EnumerateAll => None,
            SearchMode::EnumerateUpTo(limit) => Some(limit),
        }
    }
}

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

impl PuzzleClass {
    pub fn coarse_label(&self) -> &'static str {
        match self {
            PuzzleClass::Unsolvable => "Unsolvable",
            PuzzleClass::Unique => "Unique",
            PuzzleClass::Ambiguous(_) => "Ambiguous",
        }
    }

    pub fn detail_label(&self) -> String {
        match self {
            PuzzleClass::Unsolvable => "Unsolvable".to_string(),
            PuzzleClass::Unique => "Unique".to_string(),
            PuzzleClass::Ambiguous(n) => format!("Ambiguous({n})"),
        }
    }
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
    // Timing (nanoseconds)
    pub mask_init_time_ns: u128,
    pub heuristic_time_ns: u128,
    pub permutation_time_ns: u128,
    pub edge_build_time_ns: u128,
    pub pruning_time_ns: u128,
    pub extraction_time_ns: u128,
    pub total_time_ns: u128,
    // Memory (bytes)
    pub masks_memory_bytes: u64,
    pub permutation_memory_bytes: u64,
    pub graph_memory_bytes: u64,
    pub post_prune_memory_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct SolveReport<const N: usize> {
    pub solutions: Vec<Solution<N>>,
    pub stats: SolveStats<N>,
}
