use rand::seq::SliceRandom;
use rand::rng;
use solver::{SudokuSolver, init_logger, types::Board, dataset_parser::parse_puzzle_string};
use std::fs::File;
use std::io::Write;

const N: usize = 9;
const K: usize = 3;

fn main() {
    init_logger();
    
    let dataset_files = [
        "data/sample_easy.txt",
        "data/sample_medium.txt",
        "data/sample_hard.txt"
    ];

    std::fs::create_dir_all("results").unwrap();
    let mut csv_file = File::create("results/degradation.csv").expect("Could not create csv");
    writeln!(csv_file, "file,puzzle_id,starting_hints,hints_remaining,solutions_count,time_ms").unwrap();

    let limit = if cfg!(debug_assertions) {
        Some(1000)
    } else {
        None
    };

    for file_path in dataset_files {
        println!("Reading dataset: {}", file_path);
        let content = std::fs::read_to_string(file_path).unwrap_or_default();
        
        // Take the first 5 puzzles from each file
        for (puzzle_id, line) in content.lines().filter(|l| !l.trim().is_empty()).take(5).enumerate() {
            println!("Analyzing puzzle {} from {}", puzzle_id, file_path);
            
            let parsed_cells = parse_puzzle_string(line).expect("Invalid puzzle format");
            let mut cells = [[0u8; N]; N];
            let mut hints = Vec::new();
            
            for (i, &val) in parsed_cells.iter().enumerate() {
                let r = i / N;
                let c = i % N;
                cells[r][c] = val;
                if val != 0 {
                    hints.push((r, c));
                }
            }

            let mut board = Board::<N>::new(cells);
            let starting_hints = hints.len();
            
            let mut rng_thread = rng();
            hints.shuffle(&mut rng_thread);

            // Loop through hints, removing one at a time
            for i in (0..=hints.len()).rev() {
                let start = std::time::Instant::now();
                let mut solver = SudokuSolver::<N, K>::new(board);
                if let Some(l) = limit {
                    solver = solver.with_limit(l);
                }
                
                // Safety net: stop when hints get too low to avoid astronomical compute times
                if i < 23 {
                    println!("  Hints: {} -> Skipping (OOM prevention)", i);
                    writeln!(csv_file, "{},{},{},{},OOM_PREVENTED,{}", 
                        file_path, puzzle_id, starting_hints, i, start.elapsed().as_millis()).unwrap();
                    break; // Skip the rest of this puzzle
                }

                let report = solver.solve_with_stats();
                let num_solutions = report.solutions.len();
                let time_ms = start.elapsed().as_millis();
                
                println!("  Hints: {} -> Solutions: {} ({}ms)", i, num_solutions, time_ms);
                writeln!(csv_file, "{},{},{},{},{},{}", 
                    file_path, puzzle_id, starting_hints, i, num_solutions, time_ms).unwrap();

                // Remove a hint for the next iteration (unless we're at 0)
                if i > 0 {
                    let (r, c) = hints[hints.len() - i];
                    let mut next_cells = board.cells;
                    next_cells[r][c] = 0;
                    board = Board::<N>::new(next_cells);
                }
            }
        }
    }
    
    println!("Data generation complete. Results saved to results/degradation.csv");
}