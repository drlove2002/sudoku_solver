pub mod dataset_parser;
mod log;
mod solver;
pub mod types;
pub use solver::permutations;

pub use crate::log::init_logger;
pub use crate::solver::{SolveReport, SolveStats, SudokuSolver};
