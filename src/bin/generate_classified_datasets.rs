//! Phase 2: Generate classified Sudoku test sets for 9x9, 16x16, 25x25.
//!
//! Usage:
//!   cargo run --release --bin generate_classified_datasets -- -s 9 -n 500

use solver::{
    SudokuSolver, solver::report::PuzzleClass, types::Board,
    utils::dataset::{SudokuPuzzle, parse_csv, parse_puzzle_string},
};
use rand::prelude::*;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::panic;

struct Config { sizes: Vec<usize>, target_count: usize, output_dir: String }

#[derive(Debug, Clone, Copy)]
enum Category { VeryEasy, Easy, Medium, Hard, VeryHard, Ambiguous }

impl Category {
    fn all() -> Vec<Category> {
        vec![Category::VeryEasy, Category::Easy, Category::Medium,
             Category::Hard, Category::VeryHard, Category::Ambiguous]
    }
    fn label(&self) -> &'static str {
        match self { Category::VeryEasy => "very_easy", Category::Easy => "easy",
            Category::Medium => "medium", Category::Hard => "hard",
            Category::VeryHard => "very_hard", Category::Ambiguous => "ambiguous" }
    }
    fn clue_range(&self, board_size: usize) -> (usize, usize) {
        match board_size {
            81 => match self {
                Category::VeryEasy => (51, 80), Category::Easy => (36, 49), Category::Medium => (32, 35),
                Category::Hard => (28, 31), Category::VeryHard => (17, 27), Category::Ambiguous => (0, 80),
            },
            256 => match self {
                Category::VeryEasy => (157, 255), Category::Easy => (113, 155),
                Category::Medium => (101, 110), Category::Hard => (88, 98),
                Category::VeryHard => (54, 85), Category::Ambiguous => (0, 255),
            },
            625 => match self {
                Category::VeryEasy => (382, 624), Category::Easy => (275, 378),
                Category::Medium => (246, 270), Category::Hard => (215, 239),
                Category::VeryHard => (131, 208), Category::Ambiguous => (0, 624),
            },
            _ => (0, 9999),
        }
    }
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    unsafe { std::env::set_var("RUST_LOG", "error") };
    solver::init_logger();

    let mut sizes = vec![9, 16, 25];
    let mut n = 500;
    let mut out = "data/classified".to_string();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-s" => { if let Some(v) = args.get(i+1) { sizes = vec![v.parse().unwrap_or(9)]; i+=2; } else { i+=1; } }
            "-n" => { if let Some(v) = args.get(i+1) { n = v.parse().unwrap_or(500); i+=2; } else { i+=1; } }
            "-o" => { if let Some(v) = args.get(i+1) { out = v.clone(); i+=2; } else { i+=1; } }
            _ => i += 1,
        }
    }
    let config = Config { sizes, target_count: n, output_dir: out };
    fs::create_dir_all(&config.output_dir).unwrap();

    println!("=== Generate Datasets ===");
    for &sz in &config.sizes {
        match sz { 9 => run_9x9(&config), 16 => run_mxn::<16,4>(&config), 25 => run_mxn::<25,5>(&config), _ => {} }
    }
    println!("\nDone.");
}

// ========================================================================
// 9x9
// ========================================================================

fn run_9x9(cfg: &Config) {
    println!("\n--- 9x9 ---");
    let puzzles = parse_csv("data/sudoku-3m.csv").expect("CSV");
    println!("  {} puzzles", puzzles.len());
    let solutions: Vec<String> = puzzles.iter().map(|p| p.solution.clone()).collect();

    for cat in Category::all() {
        let path = format!("{}/9x9_{}.txt", cfg.output_dir, cat.label());
        let existing = count_lines(&path);
        if existing >= cfg.target_count { println!("  {:30} — done ({})", cat_label(9, &cat), existing); continue; }
        let need = cfg.target_count - existing;
        println!("  {:30} — need {}", cat_label(9, &cat), need);

        match cat {
            Category::Hard | Category::VeryHard => sample_3m(&puzzles, &cat, need, &path),
            Category::Ambiguous => gen_ambiguous::<9,3>(cfg, "9x9", &path),
            _ => gen_from_solutions::<9,3>(&solutions, &cat, need, &path),
        }
    }
}

// ========================================================================
// 16x16 / 25x25
// ========================================================================

fn run_mxn<const N: usize, const K: usize>(cfg: &Config) {
    let label = format!("{}x{}", N, N);
    println!("\n--- {} ---", label);
    let bank = format!("data/solutions_{}.txt", label);
    if !Path::new(&bank).exists() { println!("  No bank at {}", bank); return; }
    let solutions: Vec<String> = fs::read_to_string(&bank).unwrap_or_default()
        .lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect();
    println!("  {} solutions", solutions.len());
    if solutions.is_empty() { return; }

    for cat in Category::all() {
        let path = format!("{}/{}_{}.txt", cfg.output_dir, label, cat.label());
        let existing = count_lines(&path);
        if existing >= cfg.target_count { println!("  {:30} — done ({})", cat_label(N*K, &cat), existing); continue; }
        let need = cfg.target_count - existing;
        println!("  {:30} — need {}", cat_label(N*K, &cat), need);

        if matches!(cat, Category::Ambiguous) {
            gen_ambiguous::<N, K>(cfg, &label, &path);
        } else {
            gen_from_solutions::<N, K>(&solutions, &cat, need, &path);
        }
    }
}

// ========================================================================
// Puzzle generation — rejection sampling
// ========================================================================

fn gen_from_solutions<const N: usize, const K: usize>(
    solutions: &[String], cat: &Category, need: usize, path: &str,
) {
    let (min_c, max_c) = cat.clue_range(N * N);
    let target_clues = max_c; // start at upper bound
    // Retries per seed: more clues = easier to get unique
    let retries = if target_clues >= 40 { 20 } else if target_clues >= 30 { 60 } else { 150 };

    let file = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
    let mut w = BufWriter::new(file);
    let mut rng = rand::rng();
    let mut done = 0;
    let mut tries = 0u64;
    let mut si = 0usize;

    while done < need && si < solutions.len() * 200 {
        let sol = &solutions[si % solutions.len()];
        si += 1;
        let digits = match parse_puzzle_string(sol) { Ok(d) => d, Err(_) => continue };
        if digits.len() != N * N { continue; }
        let mut board = [[0u8; N]; N];
        for (idx, &v) in digits.iter().enumerate() { board[idx / N][idx % N] = v; }

        for _ in 0..retries {
            if done >= need { break; }
            tries += 1;
            if let Some(puz) = try_puzzle::<N, K>(&board, target_clues) {
                let s = puzzle_to_string::<N>(&puz);
                writeln!(w, "{}", s).unwrap(); w.flush().unwrap();
                done += 1;
                if done % 10 == 0 { println!("    {} ({:.1}%)", done, done as f64 / tries as f64 * 100.0); }
            }
        }
    }
    let rate = if tries > 0 { done as f64 / tries as f64 * 100.0 } else { 0.0 };
    println!("    done: {} ({:.1}% hit)", done, rate);
}

fn try_puzzle<const N: usize, const K: usize>(
    solution: &[[u8; N]; N], clues: usize,
) -> Option<[[u8; N]; N]> {
    let mut rng = rand::rng();
    let cells: Vec<(usize, usize)> = (0..N).flat_map(|r| (0..N).map(move |c| (r, c))).collect();
    let mut puz = [[0u8; N]; N];

    // Pick 'clues' random cells
    let mut idxs: Vec<usize> = (0..N*N).collect();
    idxs.shuffle(&mut rng);
    for &idx in &idxs[..clues] {
        let (r, c) = cells[idx];
        puz[r][c] = solution[r][c];
    }

    // Fast duplicate check per row/col
    for r in 0..N {
        let mut seen = [false; 64];
        for c in 0..N { let v = puz[r][c]; if v != 0 { if seen[v as usize] { return None; } seen[v as usize] = true; } }
    }
    for c in 0..N {
        let mut seen = [false; 64];
        for r in 0..N { let v = puz[r][c]; if v != 0 { if seen[v as usize] { return None; } seen[v as usize] = true; } }
    }

    if is_unique::<N, K>(&puz) { Some(puz) } else { None }
}

fn is_unique<const N: usize, const K: usize>(puzzle: &[[u8; N]; N]) -> bool {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let b = Board::<N>::new(*puzzle);
        let s = SudokuSolver::<N, K>::new(b);
        matches!(s.classify_up_to_two(), PuzzleClass::Unique)
    }));
    result.unwrap_or(false)
}

fn puzzle_to_string<const N: usize>(board: &[[u8; N]; N]) -> String {
    let mut s = String::with_capacity(N * N);
    for r in 0..N { for c in 0..N {
        let v = board[r][c];
        if v == 0 { s.push('.'); }
        else if N <= 9 { s.push((b'0' + v) as char); }
        else { s.push((b'A' + v - 1) as char); }
    }}
    s
}

// ========================================================================
// 9x9: sample from 3M
// ========================================================================

fn sample_3m(puzzles: &[SudokuPuzzle], cat: &Category, need: usize, path: &str) {
    let (min_c, max_c) = cat.clue_range(81);
    let matching: Vec<&SudokuPuzzle> = puzzles.iter().filter(|p| p.clues >= min_c && p.clues <= max_c).collect();
    println!("    {} matching", matching.len());
    let mut rng = rand::rng();
    let file = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
    let mut w = BufWriter::new(file);
    let take = need.min(matching.len());
    for _ in 0..take {
        let idx = rng.random_range(0..matching.len());
        writeln!(w, "{}", matching[idx].puzzle).unwrap();
    }
    println!("    wrote {}", take);
}

// ========================================================================
// Ambiguous generation: strip 2-3 clues from Very Easy puzzles
// ========================================================================

fn gen_ambiguous<const N: usize, const K: usize>(cfg: &Config, prefix: &str, path: &str) {
    let src = format!("{}/{}_very_easy.txt", cfg.output_dir, prefix);
    if !Path::new(&src).exists() { println!("    no source"); return; }
    let existing = count_lines(path);
    if existing >= cfg.target_count { println!("    done ({})", existing); return; }

    let puzzles: Vec<String> = fs::read_to_string(&src).unwrap_or_default()
        .lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect();
    println!("    stripping from {} puzzles", puzzles.len());

    let mut rng = rand::rng();
    let file = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
    let mut w = BufWriter::new(file);
    let need = cfg.target_count - existing;

    for i in 0..need.min(puzzles.len()) {
        let p = &puzzles[i % puzzles.len()];
        let mut chars: Vec<char> = p.chars().collect();
        let clue_pos: Vec<usize> = chars.iter().enumerate()
            .filter(|(_, c)| **c != '.').map(|(i, _)| i).collect();
        if clue_pos.len() < 3 { continue; }
        let picks: Vec<usize> = clue_pos.choose_multiple(&mut rng, 3).cloned().collect();
        let rm = 2 + rng.random_range(0..2); // 2 or 3
        for idx in &picks[..rm] { chars[*idx] = '.'; }
        writeln!(w, "{}", chars.into_iter().collect::<String>()).unwrap();
        w.flush().unwrap();
    }
    println!("    wrote {}", need.min(puzzles.len()));
}

// ========================================================================
// Helpers
// ========================================================================

fn count_lines(path: &str) -> usize {
    if !Path::new(path).exists() { return 0; }
    fs::read_to_string(path).unwrap_or_default().lines().filter(|l| !l.is_empty()).count()
}

fn cat_label(size: usize, cat: &Category) -> String {
    format!("{}x{}[{}]", size, size, match cat {
        Category::VeryEasy => "Very Easy", Category::Easy => "Easy",
        Category::Medium => "Medium", Category::Hard => "Hard",
        Category::VeryHard => "Very Hard", Category::Ambiguous => "Ambiguous",
    })
}
