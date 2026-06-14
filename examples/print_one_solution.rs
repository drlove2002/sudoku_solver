use sudoku_solver::{SudokuSolver, types::Board, types::masks::Masks};
use sudoku_solver::solver::{heuristics, permutations, extraction};
use sudoku_solver::types::graph::Graph;
use sudoku_solver::solver::pruning::Pruner;
use sudoku_solver::solver::report::SearchMode;

fn main() {
    const N: usize = 9;
    const K: usize = 3;
    let puzzle: [[u8; N]; N] = [
        [8,0,0,0,0,7,0,9,0],
        [0,0,0,9,4,0,0,0,5],
        [0,0,3,0,8,0,0,7,0],
        [0,0,0,5,0,0,4,0,8],
        [5,6,0,0,0,0,0,1,0],
        [0,0,0,0,6,1,0,3,0],
        [0,0,8,0,7,0,6,0,0],
        [0,5,0,3,0,0,9,0,0],
        [1,0,0,0,0,0,0,0,0],
    ];
    let board = Board::<N>::new(puzzle.clone());
    let solver = SudokuSolver::<N, K>::new(board);
    let report = solver.solve_with_stats();
    
    if let Some(sol) = report.solutions.first() {
        println!("BOARD_AFTER_HEURISTICS:");
        // To get this we need the heuristics board... 
        // Let me just output the solution
        for r in 0..N {
            println!("[{:?}],", sol.board.cells[r]);
        }
    }
}
