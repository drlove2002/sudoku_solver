use solver::{SudokuSolver, init_logger, types::Board, utils::dataset::parse_puzzle_string};

const N: usize = 9;
const K: usize = 3;

fn main() {
    init_logger();

    // Test with a Kaggle-format puzzle
    let puzzle_str =
        "...81.....2........1.9..7...7..25.934.2............5...975.....563.....4......68.";

    // Parse Kaggle format
    let cells_vec = parse_puzzle_string(puzzle_str).expect("Failed to parse puzzle");

    // Convert to 2D array
    let mut cells = [[0u8; N]; N];
    for (i, &val) in cells_vec.iter().enumerate() {
        cells[i / N][i % N] = val;
    }

    println!("Input puzzle:");
    let board = Board::<N>::new(cells);
    println!("{}", board);

    let solver = SudokuSolver::<N, K>::new(board);
    let solutions = solver.solve();

    println!("\nFound {} solution(s)", solutions.len());

    for (idx, solution) in solutions.iter().enumerate() {
        println!("\nSolution {}:", idx + 1);
        println!("{}", solution);
    }
}
