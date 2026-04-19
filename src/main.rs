use log::{debug, info};
use solver::{utils::dataset::parse_puzzle_string, SudokuSolver, init_logger, types};

const N: usize = 9;
const K: usize = N.isqrt();

fn main() {
    init_logger();
    info!("Starting Sudoku Solver");

    let args: Vec<String> = std::env::args().collect();
    
    // Parse arguments
    let visualize = args.contains(&"--visualize".to_string());
    
    let input_file = if args.len() > 1 && !args[1].starts_with("--") {
        &args[1]
    } else {
        "dataset/simple_test.txt"
    };

    let content = std::fs::read_to_string(input_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", input_file));
    info!("Read input file: {}", input_file);

    // Clean up content (remove spaces, newlines, tabs)
    let cleaned_content: String = content
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '0' { '.' } else { c })
        .collect();

    let parsed_cells = parse_puzzle_string(&cleaned_content)
        .unwrap_or_else(|e| panic!("Failed to parse puzzle: {}", e));

    let mut cells = [[0u8; N]; N];
    for (i, &val) in parsed_cells.iter().enumerate() {
        cells[i / N][i % N] = val;
    }

    let board = types::Board::<N>::new(cells);
    info!("Board created successfully");
    debug!("Board state:\n{}", board);

    let mut solver = SudokuSolver::<N, K>::new(board);
    if cfg!(debug_assertions) {
        solver = solver.with_limit(1000);
    }
    if visualize {
        solver = solver.with_visualize(true);
    }
    info!("Solver initialized");

    let solutions = solver.solve();
    info!("Solving completed - found {} solution(s)", solutions.len());
}
