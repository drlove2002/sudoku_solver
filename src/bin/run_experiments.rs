//! Phase 3: Run all classified puzzles through solver with/without heuristics.
//! Checkpoint after each puzzle. On crash/restart, resume from checkpoint.
//!
//! Output: results/experiment_results.csv  — all times in nanoseconds
//!
//! Usage:
//!   cargo run --release --bin run_experiments
//!   cargo run --release --bin run_experiments -- --size 9
//!   cargo run --release --bin run_experiments -- --no-heuristics

use log::LevelFilter;
use solver::{SudokuSolver, types::Board, utils::dataset::parse_puzzle_string};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write, Seek, SeekFrom};
use std::panic;
use std::path::Path;

const RESULTS_CSV: &str = "results/experiment_results.csv";
const DATA_DIR: &str = "data/classified";
const BREADCRUMB_DIR: &str = "results/breadcrumbs";

fn main() {
    panic::set_hook(Box::new(|_| {}));
    solver::init_logger_with_level(LevelFilter::Error);

    let args: Vec<String> = std::env::args().collect();
    let size_filter: Option<usize> = args.iter()
        .position(|a| a == "--size").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok());
    let heuristics_only = args.contains(&"--heuristics-only".to_string());
    let no_heuristics = args.contains(&"--no-heuristics".to_string());

    fs::create_dir_all("results").ok();
    fs::create_dir_all(BREADCRUMB_DIR).ok();

    // Phase 0: write header + flush OOM breadcrumbs from prior run
    let had_header = Path::new(RESULTS_CSV).exists();
    {
        let file0 = OpenOptions::new().create(true).append(true).open(RESULTS_CSV).unwrap();
        let mut writer0 = BufWriter::new(file0);
        if !had_header {
            writeln!(writer0, "size,category,puzzle_idx,heuristic_on,clues,classification,cells_filled,\
                mask_ns,heuristic_ns,perm_ns,graph_ns,prune_ns,extract_ns,total_ns,\
                mask_mem,heuristic_mem,perm_mem,graph_mem,prune_mem,\
                initial_vertices,initial_edges,pruned_vertices,pruned_edges,removed_vertices,\
                permutation_counts,phase_progress"
            ).unwrap();
        }
        flush_oom_breadcrumbs(&mut writer0);
    }

    let completed = load_checkpoint();

    let file = OpenOptions::new().create(true).append(true).open(RESULTS_CSV).unwrap();
    let mut writer = BufWriter::new(file);

    let categories = ["very_easy", "easy", "medium", "hard", "very_hard", "ambiguous"];

    for &size in &[9, 16, 25] {
        if let Some(sf) = size_filter { if size != sf { continue; } }
        let label = format!("{}x{}", size, size);

        for &cat in &categories {
            let path = format!("{}/{}_{}.txt", DATA_DIR, label, cat);
            if !Path::new(&path).exists() { continue; }
            let puzzles: Vec<String> = fs::read_to_string(&path).unwrap_or_default()
                .lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect();
            println!("\n{} {}: {} puzzles", label, cat, puzzles.len());

            for heuristic_on in [true, false] {
                if (heuristics_only && !heuristic_on) || (no_heuristics && heuristic_on) { continue; }
                let h_label = if heuristic_on { "on" } else { "off" };
                let h_label_pad = if heuristic_on { "on " } else { "off" };

                for (idx, puzzle) in puzzles.iter().enumerate() {
                    let key = (label.clone(), cat.to_string(), idx, heuristic_on);
                    if completed.contains(&key) { continue; }

                    print!("  [{:>3}] h={} ", idx, h_label_pad);
                    let line = match size {
                        9 => run_one::<9, 3>(puzzle, heuristic_on, &label, cat, idx),
                        16 => run_one::<16, 4>(puzzle, heuristic_on, &label, cat, idx),
                        25 => run_one::<25, 5>(puzzle, heuristic_on, &label, cat, idx),
                        _ => continue,
                    };

                    if let Some(l) = line {
                        writeln!(writer, "{}", l).unwrap();
                        writer.flush().unwrap();
                    }
                }
            }
        }
    }

    println!("\nDone. See {RESULTS_CSV}");
}

fn run_one<const N: usize, const K: usize>(
    puzzle_str: &str, heuristic_on: bool,
    size_label: &str, cat: &str, idx: usize,
) -> Option<String> {
    let digits = match parse_puzzle_string(puzzle_str) { Ok(d) => d, Err(e) => { println!("PARSE: {e}"); return None; } };
    if digits.len() != N * N { println!("LEN: {}", digits.len()); return None; }

    let mut cells = [[0u8; N]; N];
    for (i, &v) in digits.iter().enumerate() { cells[i / N][i % N] = v; }
    let clues = cells.iter().flatten().filter(|&&c| c != 0).count();

    let board = Board::<N>::new(cells);
    let h_label = if heuristic_on { "on" } else { "off" };
    let breadcrumb_file = format!("{BREADCRUMB_DIR}/{size_label}_{cat}_{idx}_h{h_label}.txt");
    std::fs::create_dir_all(BREADCRUMB_DIR).ok();

    let solver = if heuristic_on {
        SudokuSolver::<N, K>::new(board).with_breadcrumb(&breadcrumb_file)
    } else {
        SudokuSolver::<N, K>::new(board).without_heuristics().with_breadcrumb(&breadcrumb_file)
    };

    let report = match panic::catch_unwind(panic::AssertUnwindSafe(|| solver.solve_with_stats())) {
        Ok(r) => r,
        Err(_) => {
            println!("PANIC");
            return Some(skip_row(size_label, cat, idx, heuristic_on, clues));
        }
    };

    let s = &report.stats;
    let perm_str: String = s.permutation_counts.iter()
        .map(|c| c.to_string()).collect::<Vec<_>>().join(";");
    let class = s.puzzle_classification.detail_label();
    let total_us = s.total_time_ns as f64 / 1000.0;
    let progress = format!("{:?}", s.phase_progress);
    println!("OK ({:.1}us, {}, perm={}/{})", total_us, class, s.pruned_vertex_count, s.initial_vertex_count);

    // Delete breadcrumb on success
    let _ = std::fs::remove_file(&breadcrumb_file);

    Some(format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        size_label, cat, idx, heuristic_on, clues, class, s.heuristic_cells_filled,
        s.mask_init_time_ns, s.heuristic_time_ns, s.permutation_time_ns,
        s.edge_build_time_ns, s.pruning_time_ns, s.extraction_time_ns, s.total_time_ns,
        s.masks_memory_bytes, s.heuristic_memory_bytes,
        s.permutation_memory_bytes, s.graph_memory_bytes, s.post_prune_memory_bytes,
        s.initial_vertex_count, s.initial_edge_count,
        s.pruned_vertex_count, s.pruned_edge_count, s.removed_vertices,
        perm_str, progress,
    ))
}

fn skip_row(size: &str, cat: &str, idx: usize, h: bool, clues: usize) -> String {
    format!("{},{},{},{},{},Panic,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,,Panic", size, cat, idx, h, clues)
}

// ---------------------------------------------------------------------------

type CheckpointKey = (String, String, usize, bool);

fn load_checkpoint() -> HashSet<CheckpointKey> {
    let mut set = HashSet::new();
    if !Path::new(RESULTS_CSV).exists() { return set; }
    let file = File::open(RESULTS_CSV).unwrap();
    for line in BufReader::new(file).lines().skip(1) {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                set.insert((
                    parts[0].to_string(), parts[1].to_string(),
                    parts[2].parse().unwrap_or(0), parts[3].parse().unwrap_or(false),
                ));
            }
        }
    }
    set
}

// On restart: scan breadcrumb files. If a breadcrumb exists for a puzzle NOT in the CSV,
// it OOM'd. Write the OOM row to CSV and delete the breadcrumb.
fn flush_oom_breadcrumbs(writer: &mut BufWriter<File>) {
    // Load existing CSV rows to know what's already done
    let completed = load_checkpoint();
    let dir = match std::fs::read_dir(BREADCRUMB_DIR) {
        Ok(d) => d,
        Err(_) => return,
    };

    let mut oom_rows: Vec<String> = Vec::new();
    let mut stale_count = 0usize;
    for entry in dir {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let fname = entry.file_name().to_string_lossy().to_string();
        if !fname.ends_with(".txt") { continue; }

        // Parse: {size}x{size}_{category}_{idx}_h{on/off}.txt
        // Category may contain underscores (e.g., "very_easy")
        // So we parse from the right: last 3 parts are idx, hflag, then everything before is size+cat
        let stem = fname.strip_suffix(".txt").unwrap_or(&fname);
        let parts: Vec<&str> = stem.rsplitn(3, '_').collect();
        // rsplitn from right: ["hoff" or "hon", idx_str, rest...]
        if parts.len() < 3 { continue; }
        let h_on = parts[0] == "hon";
        let idx: usize = match parts[1].parse() { Ok(i) => i, Err(_) => continue };
        // parts[2] is "{size}x{size}_{category}"
        // Split on first underscore to get size_label and category
        let rest = parts[2];
        let (size_label, cat) = if let Some(pos) = rest.find('_') {
            (rest[..pos].to_string(), rest[pos+1..].to_string())
        } else {
            continue;
        };

        // Skip if already in CSV — clean up stale breadcrumb
        if completed.contains(&(size_label.clone(), cat.clone(), idx, h_on)) {
            let _ = std::fs::remove_file(entry.path());
            stale_count += 1;
            continue;
        }

        // Read breadcrumb to find which phase was reached
        let phase_reached = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        let oom_phase = match phase_reached.as_str() {
            "permutations" => "OomAt(permutation_generation)",
            "graph" => "OomAt(graph_construction)",
            "pruning" => "OomAt(pruning)",
            "extraction" => "OomAt(extraction)",
            _ => "OomAt(unknown)",
        };

        // Load clue count from puzzle file
        let puzzle_path = format!("{DATA_DIR}/{size_label}_{cat}.txt");
        let clue_count = load_clue_count(&puzzle_path, idx);
        oom_rows.push(format!(
            "{},{},{},{},{},OOM,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,,{oom_phase}",
            size_label, cat, idx, h_on, clue_count
        ));

        let _ = std::fs::remove_file(entry.path());
    }

    for row in &oom_rows {
        writeln!(writer, "{}", row).unwrap();
    }
    if !oom_rows.is_empty() {
        writer.flush().unwrap();
        println!("  Flushed {} OOM breadcrumbs to CSV", oom_rows.len());
    }
}

fn load_clue_count(path: &str, idx: usize) -> String {
    if let Ok(content) = std::fs::read_to_string(path) {
        let line = content.lines().nth(idx).unwrap_or("");
        format!("{}", line.chars().filter(|&c| c != '.' && c != '0').count())
    } else {
        "0".to_string()
    }
}
