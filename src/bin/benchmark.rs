use solver::{SudokuSolver, dataset_parser::parse_puzzle_string, init_logger, types::Board};
use std::fs::File;
use std::io::Write;
use std::time::Duration;

const N: usize = 9;
const K: usize = 3;

fn main() {
    init_logger();

    let datasets = vec![
        ("unique", "easy"),
        ("unique", "medium"),
        ("unique", "hard"),
        ("ambiguous", "easy"),
        ("ambiguous", "medium"),
        ("ambiguous", "hard"),
        ("none", "easy"),
        ("none", "medium"),
        ("none", "hard"),
    ];

    std::fs::create_dir_all("results").unwrap();
    let mut csv_file = File::create("results/benchmark_timings.csv").unwrap();
    writeln!(
        csv_file,
        "puzzle_type,difficulty,clues_count,mask_init_ns,heuristic_ns,permutation_ns,edge_build_ns,pruning_ns,extraction_ns,total_ns,solution_count"
    )
    .unwrap();

    println!("Starting Benchmark Run...");
    println!("Make sure to run this with `cargo run --release --bin benchmark`");

    let total_start = std::time::Instant::now();

    for (ptype, diff) in datasets {
        let filename = format!("data/{}_{}.txt", ptype, diff);
        println!("Processing {}...", filename);

        let content = std::fs::read_to_string(&filename).expect("Dataset file missing");

        for (i, line) in content.lines().filter(|l| !l.trim().is_empty()).enumerate() {
            let parsed_cells = parse_puzzle_string(line).expect("Invalid format");
            let mut cells = [[0u8; N]; N];
            let mut clues = 0;

            for (idx, &val) in parsed_cells.iter().enumerate() {
                cells[idx / N][idx % N] = val;
                if val != 0 {
                    clues += 1;
                }
            }

            let board = Board::<N>::new(cells);

            let solver = SudokuSolver::<N, K>::new(board).with_limit(10_000);
            let report = solver.solve_with_stats();

            writeln!(
                csv_file,
                "{},{},{},{},{},{},{},{},{},{},{}",
                ptype,
                diff,
                clues,
                report.stats.mask_init_time_ns,
                report.stats.heuristic_time_ns,
                report.stats.permutation_time_ns,
                report.stats.edge_build_time_ns,
                report.stats.pruning_time_ns,
                report.stats.extraction_time_ns,
                report.stats.total_time_ns,
                report.stats.solution_count
            )
            .unwrap();

            if report.stats.total_time_ns > 10_000_000_000 {
                println!(
                    "  [WARN] Puzzle {} took >10s ({}s). Hard explosion detected. Skipping.",
                    i,
                    Duration::from_nanos(report.stats.total_time_ns as u64).as_secs_f32()
                );
            }
        }
    }

    println!("Benchmark Complete in {:.2}s. Data written to results/benchmark_timings.csv", total_start.elapsed().as_secs_f32());
}
