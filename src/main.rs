use log::{debug, info};
use solver::{SudokuSolver, init_logger, types};

fn main() {
    init_logger();
    info!("Starting Sudoku Solver");

    let content = std::fs::read_to_string("dataset/input.txt").expect("Failed to read input.txt");
    info!("Read input file successfully");

    let mut cells = [[0u8; 9]; 9];
    let mut nums = content
        .split_whitespace()
        .map(|s| s.parse::<u8>().expect("Invalid number"));

    for row in cells.iter_mut() {
        for cell in row.iter_mut() {
            *cell = nums.next().expect("Not enough numbers in input file");
        }
    }

    let board = types::Board::<9>::new(cells);
    info!("Board created successfully");
    debug!("Board state:\n{}", board);

    let solver = SudokuSolver::<9>::new(board);
    info!("Solver initialized");

    let solution = solver.solve();
    info!("Solving completed");

    // Print number of permutations generated for each minigrid
    for (i, perms) in solution.iter().enumerate() {
        println!("Minigrid {}: {} permutations", i + 1, perms.len());
    }
}
