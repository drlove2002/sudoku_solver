use log::{debug, info};
use solver::{SudokuSolver, init_logger, types};

const N: usize = 9;
const K: usize = N.isqrt();

fn main() {
    init_logger();
    info!("Starting Sudoku Solver");

    let args: Vec<String> = std::env::args().collect();
    let input_file = if args.len() > 1 {
        &args[1]
    } else {
        "dataset/simple_test.txt"
    };

    let content = std::fs::read_to_string(input_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", input_file));
    info!("Read input file: {}", input_file);

    let mut cells = [[0u8; N]; N];
    let mut nums = content
        .split_whitespace()
        .map(|s| s.parse::<u8>().expect("Invalid number"));

    for row in cells.iter_mut() {
        for cell in row.iter_mut() {
            *cell = nums.next().expect("Not enough numbers in input file");
        }
    }

    let board = types::Board::<N>::new(cells);
    info!("Board created successfully");
    debug!("Board state:\n{}", board);

    let mut solver = SudokuSolver::<N, K>::new(board);
    if cfg!(debug_assertions) {
        solver = solver.with_limit(1000);
    }
    info!("Solver initialized");

    let solutions = solver.solve();
    info!("Solving completed - found {} solution(s)", solutions.len());
}
