//! Phase 1: Solve all 16x16 and 25x25 instances from SAMPLE INSTANCES.txt
//! and write their solutions as seed banks for dataset generation.
//!
//! Usage:
//!   cargo run --release --bin generate_solution_bank              # both sizes
//!   cargo run --release --bin generate_solution_bank -- 16        # 16x16 only
//!   cargo run --release --bin generate_solution_bank -- 25        # 25x25 only
//!   cargo run --release --bin generate_solution_bank -- 16 --resume  # skip if file exists
//!
//! Outputs:
//!   data/solutions_16x16.txt — one solution per line (A-P chars, 256 chars)
//!   data/solutions_25x25.txt — one solution per line (A-Y chars, 625 chars)
//!   data/bank_manifest.txt  — summary: puzzle count, solve status per instance

use solver::{SudokuSolver, types::Board, utils::dataset::parse_puzzle_string};
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::panic;
use std::path::Path;

fn main() {
    panic::set_hook(Box::new(|_| {}));
    solver::init_logger();

    let args: Vec<String> = std::env::args().collect();
    let size_filter: Option<usize> = args.get(1).and_then(|s| s.parse().ok());

    let content = fs::read_to_string("data/SAMPLE INSTANCES.txt")
        .expect("Failed to read data/SAMPLE INSTANCES.txt");
    let instances = parse_sample_instances(&content);

    let mut size_16: Vec<Instance> = Vec::new();
    let mut size_25: Vec<Instance> = Vec::new();
    for inst in &instances {
        match inst.puzzle_len() {
            256 => size_16.push(inst.clone()),
            625 => size_25.push(inst.clone()),
            _ => {}
        }
    }

    fs::create_dir_all("data").ok();

    if size_filter.map_or(true, |s| s == 16) {
        solve_and_write::<16, 4>(&size_16, "data/solutions_16x16.txt");
    }
    if size_filter.map_or(true, |s| s == 25) {
        solve_and_write::<25, 5>(&size_25, "data/solutions_25x25.txt");
    }

    // Write manifest
    write_manifest::<16, 4>(&size_16, "data/solutions_16x16.txt");
    write_manifest::<25, 5>(&size_25, "data/solutions_25x25.txt");

    println!("\nDone.");
}

fn solve_and_write<const N: usize, const K: usize>(instances: &[Instance], output_path: &str) {
    let size_label = match N {
        16 => "16x16",
        25 => "25x25",
        _ => "unknown",
    };
    let alphabet = |d: u8| -> char {
        if d == 0 { '.' } else { (b'A' + d - 1) as char }
    };

    // Count existing solutions for resume support
    let existing_solutions = if Path::new(output_path).exists() {
        fs::read_to_string(output_path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty())
            .count()
    } else {
        0
    };

    println!("\n--- {}: {} instances ({} already solved) ---", size_label, instances.len(), existing_solutions);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)
        .expect("Failed to open solutions file");
    let mut writer = BufWriter::new(file);

    let mut total_ok = existing_solutions;
    let mut total_panic = 0;
    let mut total_none = 0;
    let mut skipped = 0;

    for (i, inst) in instances.iter().enumerate() {
        // Skip already-solved instances (count matches position in file)
        if i < existing_solutions {
            skipped += 1;
            continue;
        }
        print!("  [{}] {}... ", i + 1, inst.category);

        let digits = match parse_puzzle_string(&inst.puzzle) {
            Ok(d) => d,
            Err(e) => {
                println!("PARSE ERROR: {}", e);
                continue;
            }
        };

        if digits.len() != N * N {
            println!("SKIP (len={})", digits.len());
            continue;
        }

        let mut cells = [[0u8; N]; N];
        for (idx, &val) in digits.iter().enumerate() {
            cells[idx / N][idx % N] = val;
        }
        let board = Board::<N>::new(cells);

        let solver = SudokuSolver::<N, K>::new(board);
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| solver.solve_one()));

        match result {
            Ok(Some(solution)) => {
                let mut sol_str = String::with_capacity(N * N);
                for r in 0..N {
                    for c in 0..N {
                        sol_str.push(alphabet(solution.cells[r][c]));
                    }
                }
                writeln!(writer, "{}", sol_str).unwrap();
                writer.flush().unwrap(); // flush immediately for OOM resilience
                let clues = inst.puzzle.chars().filter(|&c| c != '.' && c != '0').count();
                println!("OK ({} clues)", clues);
                total_ok += 1;
            }
            Ok(None) => {
                let class = solver.classify_up_to_two();
                println!("NONE ({})", class.detail_label());
                total_none += 1;
            }
            Err(_) => {
                println!("INVALID BOARD (duplicate clue)");
                total_panic += 1;
            }
        }
    }

    writer.flush().unwrap();
    println!(
        "  {} solutions total ({} skipped, {} new ok, {} unsolvable, {} invalid)",
        total_ok, skipped, total_ok - skipped, total_none, total_panic
    );
}

fn write_manifest<const N: usize, const K: usize>(instances: &[Instance], solutions_path: &str) {
    let solutions: Vec<String> = if Path::new(solutions_path).exists() {
        fs::read_to_string(solutions_path)
            .unwrap_or_default()
            .lines()
            .map(|l| l.to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    let mut manifest = String::new();
    let size_label = format!("{N}x{N}");
    manifest.push_str(&format!("{}: {} instances, {} solutions\n\n", size_label, instances.len(), solutions.len()));

    let mut sol_iter = solutions.iter();
    for (i, inst) in instances.iter().enumerate() {
        let clues = inst.puzzle.chars().filter(|&c| c != '.' && c != '0').count();
        let status = match sol_iter.next() {
            Some(_) => "solved".to_string(),
            None => "failed".to_string(),
        };
        manifest.push_str(&format!(
            "  [{:02}] {} | status={} | clues={}\n",
            i + 1, inst.category, status, clues
        ));
    }

    // Append to manifest (both sizes)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("data/bank_manifest.txt")
        .expect("Failed to open manifest");
    write!(file, "\n{}\n", manifest).unwrap();
}

// ---------------------------------------------------------------------------
// Instance parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Instance {
    category: String,
    puzzle: String,
}

impl Instance {
    fn puzzle_len(&self) -> usize {
        self.puzzle.len()
    }
}

fn parse_sample_instances(content: &str) -> Vec<Instance> {
    let mut instances = Vec::new();
    let mut current_category = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();
        if lower.contains("sudoku") {
            current_category = line.to_string();
            continue;
        }
        if let Some(dot_pos) = line.find('.') {
            let prefix = &line[..dot_pos];
            if prefix.chars().all(|c| c.is_ascii_digit() || c.is_whitespace()) {
                let puzzle = line[dot_pos + 1..].trim().to_string();
                if !puzzle.is_empty() {
                    instances.push(Instance {
                        category: current_category.clone(),
                        puzzle,
                    });
                }
            }
        }
    }

    instances
}
