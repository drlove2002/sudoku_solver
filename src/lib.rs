pub mod dataset_parser;
pub mod helper;
mod log;
mod solver;
pub mod types;
pub use solver::permutations;

pub use crate::log::init_logger;
pub use crate::solver::{SolveReport, SolveStats, SudokuSolver};
