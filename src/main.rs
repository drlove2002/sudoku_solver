use log::{debug, info};
use solver::{SudokuSolver, init_logger, types, utils::dataset::parse_puzzle_string};

fn run_puzzle<const N: usize, const K: usize>(parsed_cells: &[u8], visualize: bool) {
    let mut cells = [[0u8; N]; N];
    for (i, &val) in parsed_cells.iter().enumerate() {
        cells[i / N][i % N] = val;
    }

    let board = types::Board::<N>::new(cells);
    info!("Board created successfully");
    println!("--- Parsed Board ---");
    println!("{}", board);
    debug!("Board state:\n{}", board);

    let mut solver = SudokuSolver::<N, K>::new(board);
    if cfg!(debug_assertions) {
        solver = solver.with_limit(1000);
    }
    if visualize {
        solver = solver.with_visualize(true);
    }
    info!("Solver initialized");

    let report = solver.solve_with_stats();
    info!("Solving completed - found {} solution(s)", report.solutions.len());
    
    println!("\n=== PERFORMANCE TIMINGS ===");
    println!("Mask Init:       {:.2} ms", report.stats.mask_init_time_ns as f64 / 1_000_000.0);
    println!("Heuristics:      {:.2} ms", report.stats.heuristic_time_ns as f64 / 1_000_000.0);
    println!("Permutations:    {:.2} ms", report.stats.permutation_time_ns as f64 / 1_000_000.0);
    println!("Edge Building:   {:.2} ms", report.stats.edge_build_time_ns as f64 / 1_000_000.0);
    println!("Pruning:         {:.2} ms", report.stats.pruning_time_ns as f64 / 1_000_000.0);
    println!("Extraction:      {:.2} ms", report.stats.extraction_time_ns as f64 / 1_000_000.0);
    println!("Total Time:      {:.2} ms", report.stats.total_time_ns as f64 / 1_000_000.0);
}

fn main() {
    init_logger();
    info!("Starting Sudoku Solver");

    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let visualize = args.contains(&"--visualize".to_string());

    let input_str = if args.len() > 1 && !args[1].starts_with("--") {
        &args[1]
    } else {
        "dataset/simple_test.txt"
    };

    let content = if std::path::Path::new(input_str).exists() {
        info!("Read input file: {}", input_str);
        std::fs::read_to_string(input_str)
            .unwrap_or_else(|_| panic!("Failed to read {}", input_str))
    } else {
        info!("Parsing input as direct puzzle string");
        input_str.to_string()
    };

    // Clean up content (remove spaces, newlines, tabs)
    let cleaned_content: String = content
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '0' { '.' } else { c })
        .collect();

    let parsed_cells = parse_puzzle_string(&cleaned_content)
        .unwrap_or_else(|e| panic!("Failed to parse puzzle: {}", e));

    match parsed_cells.len() {
        81 => run_puzzle::<9, 3>(&parsed_cells, visualize),
        256 => run_puzzle::<16, 4>(&parsed_cells, visualize),
        625 => run_puzzle::<25, 5>(&parsed_cells, visualize),
        len => panic!("Unsupported puzzle length: {}", len),
    }
}
