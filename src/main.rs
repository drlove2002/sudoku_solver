use log::{debug, info};
use solver::{SudokuSolver, init_logger, types, utils::dataset::parse_puzzle_string};

fn run_puzzle<const N: usize, const K: usize>(parsed_cells: &[u8], visualize: Option<String>, no_heuristics: bool) {
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
    if let Some(path) = visualize {
        solver = solver.with_visualize(path);
    }
    if no_heuristics {
        solver = solver.without_heuristics();
    }
    info!("Solver initialized");

    let report = solver.solve_with_stats();
    info!(
        "Solving completed - found {} solution(s)",
        report.solutions.len()
    );

    println!("\n=== PERFORMANCE TIMINGS ===");
    println!(
        "Mask Init:       {:.2} ms",
        report.stats.mask_init_time_ns as f64 / 1_000_000.0
    );
    println!(
        "Heuristics:      {:.2} ms",
        report.stats.heuristic_time_ns as f64 / 1_000_000.0
    );
    println!(
        "Permutations:    {:.2} ms",
        report.stats.permutation_time_ns as f64 / 1_000_000.0
    );
    println!(
        "Edge Building:   {:.2} ms",
        report.stats.edge_build_time_ns as f64 / 1_000_000.0
    );
    println!(
        "Pruning:         {:.2} ms",
        report.stats.pruning_time_ns as f64 / 1_000_000.0
    );
    println!(
        "Extraction:      {:.2} ms",
        report.stats.extraction_time_ns as f64 / 1_000_000.0
    );
    println!(
        "Total Time:      {:.2} ms",
        report.stats.total_time_ns as f64 / 1_000_000.0
    );

    fn mem_str(bytes: u64) -> String {
        if bytes >= 1_048_576 {
            format!("{:.1} MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    let total_pre = report.stats.masks_memory_bytes
        + report.stats.permutation_memory_bytes
        + report.stats.graph_memory_bytes;
    println!("\n=== MEMORY FOOTPRINT ===");
    println!(
        "Masks + Board:   {}",
        mem_str(report.stats.masks_memory_bytes)
    );
    println!(
        "Permutations:    {}",
        mem_str(report.stats.permutation_memory_bytes)
    );
    println!(
        "Graph:           {}",
        mem_str(report.stats.graph_memory_bytes)
    );
    println!("────────────────────────");
    println!(
        "Total (pre-prune): {}",
        mem_str(total_pre)
    );
    println!(
        "Post-prune:      {}",
        mem_str(report.stats.post_prune_memory_bytes)
    );
}

fn main() {
    init_logger();
    info!("Starting Sudoku Solver");

    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let visualize_output = args.iter().find(|a| *a == "--visualize" || a.starts_with("--visualize="));
    let visualize_path = visualize_output.and_then(|a| {
        if let Some((_, path)) = a.split_once('=') {
            Some(path.to_string())
        } else {
            None
        }
    });
    let no_heuristics = args.contains(&"--no-heuristics".to_string());

    // Find the first positional arg (skip args[0] which is the binary path)
    let input_pos = args.iter().skip(1).position(|a| !a.starts_with("--"));
    let input_str = if let Some(pos) = input_pos {
        &args[pos + 1]
    } else {
        "dataset/simple_test.txt"
    };
    let mut default_output_path = std::path::Path::new(input_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|stem| format!("results/graph_{}_9x9.json", stem));
    if no_heuristics && visualize_path.is_none() {
        default_output_path = default_output_path.map(|p| p.replace(".json", "_no_heuristics.json"));
    }

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

    let graph_output = visualize_path.or(default_output_path);
    match parsed_cells.len() {
        81 => run_puzzle::<9, 3>(&parsed_cells, graph_output, no_heuristics),
        256 => run_puzzle::<16, 4>(&parsed_cells, graph_output, no_heuristics),
        625 => run_puzzle::<25, 5>(&parsed_cells, graph_output, no_heuristics),
        len => panic!("Unsupported puzzle length: {}", len),
    }
}
