pub mod solver;
pub mod types;
pub mod utils;

pub use crate::solver::SudokuSolver;
pub use crate::solver::report::{SearchMode, SolveReport, SolveStats};
pub use crate::utils::init_logger;
pub use solver::permutations;
