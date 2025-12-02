mod solver;
mod types;

// Craete test module
#[cfg(test)]
mod test_sudoku {
    // Import the necessary items from the parent module
    use super::*;
    #[test]
    fn test_example() {
        let cells = [
            [7, 4, 5, 0, 9, 0, 0, 0, 0],
            [0, 3, 2, 1, 5, 0, 0, 4, 6],
            [0, 0, 0, 2, 8, 0, 5, 0, 3],
            [2, 0, 0, 0, 0, 0, 0, 6, 0],
            [9, 8, 0, 6, 0, 0, 3, 5, 1],
            [0, 0, 0, 5, 4, 0, 2, 0, 7],
            [3, 0, 8, 0, 0, 0, 0, 0, 2],
            [0, 2, 0, 7, 6, 0, 0, 1, 0],
            [0, 6, 0, 9, 0, 8, 0, 3, 4],
        ];
        let flat_cells: Vec<u8> = cells.iter().flatten().copied().collect();
        let mut solver = solver::SudokuSolver::new(cells.len(), flat_cells);
        let solution = solver.solve();

        // Check if the solution is follow all sudoku rules
        for solution in solution {
            let board = types::Board {
                size: cells.len(),
                cells: solution,
            };
            assert!(board.is_valid());
        }

        // Print number of permutations generated for each minigrid
        for (i, perms) in solver.permutations.iter().enumerate() {
            println!("Minigrid {}: {} permutations", i, perms.len());
        }
    }
}
