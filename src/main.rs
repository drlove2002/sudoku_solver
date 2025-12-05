use solver::{SudokuSolver, types};

fn main() {
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

    let board = types::Board { cells };
    let mut solver = SudokuSolver::<9>::new(board);
    let solution = solver.solve();

    // Check if the solution is follow all sudoku rules
    for solution in solution {
        // Convert flat solution back to 2D array for validation
        let mut solved_cells = [[0u8; 9]; 9];
        for i in 0..81 {
            solved_cells[i / 9][i % 9] = solution[i];
        }

        let board = types::Board {
            cells: solved_cells,
        };
        assert!(board.is_valid());
    }

    // Print number of permutations generated for each minigrid
    for (i, perms) in solver.permutations.iter().enumerate() {
        println!("Minigrid {}: {} permutations", i, perms.len());
    }
}
