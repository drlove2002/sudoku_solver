use rand::rng;
use rand::seq::IndexedRandom;
use rand::seq::SliceRandom;
use solver::utils::dataset::{parse_csv, stratified_sample, SudokuPuzzle};
use std::fs::File;
use std::io::Write;

const N_SAMPLES_PER_DIFF: usize = 100;

fn main() {
    println!("Loading main dataset from data/sudoku-3m.csv...");
    let all_puzzles = parse_csv("data/sudoku-3m.csv").expect("Failed to parse main csv");

    println!("Loaded {} puzzles", all_puzzles.len());

    let sampled = stratified_sample(
        &all_puzzles,
        N_SAMPLES_PER_DIFF,
        N_SAMPLES_PER_DIFF,
        N_SAMPLES_PER_DIFF,
    );

    let easy_puzzles = &sampled[0..N_SAMPLES_PER_DIFF];
    let medium_puzzles = &sampled[N_SAMPLES_PER_DIFF..N_SAMPLES_PER_DIFF * 2];
    let hard_puzzles = &sampled[N_SAMPLES_PER_DIFF * 2..N_SAMPLES_PER_DIFF * 3];

    write_datasets("easy", easy_puzzles);
    write_datasets("medium", medium_puzzles);
    write_datasets("hard", hard_puzzles);

    println!("Dataset generation complete. Saved to data/*.txt");
}

fn write_datasets(difficulty: &str, puzzles: &[SudokuPuzzle]) {
    let mut unique_file = File::create(format!("data/unique_{}.txt", difficulty)).unwrap();
    let mut ambiguous_file = File::create(format!("data/ambiguous_{}.txt", difficulty)).unwrap();
    let mut none_file = File::create(format!("data/none_{}.txt", difficulty)).unwrap();

    let mut rng_thread = rng();

    for p in puzzles {
        // Write unique solution puzzle
        writeln!(unique_file, "{}", p.puzzle).unwrap();

        // Write ambiguous puzzle (remove 3 clues)
        let mut ambiguous_chars: Vec<char> = p.puzzle.chars().collect();
        let mut clue_indices: Vec<usize> = ambiguous_chars
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| if c != '.' && c != '0' { Some(i) } else { None })
            .collect();
        
        clue_indices.shuffle(&mut rng_thread);
        for &idx in clue_indices.iter().take(3) {
            ambiguous_chars[idx] = '.';
        }
        writeln!(ambiguous_file, "{}", ambiguous_chars.iter().collect::<String>()).unwrap();

        // Write unsolvable puzzle (find an empty cell, place an invalid digit that breaks global state)
        // To ensure it passes Phase 1 (masks), we pick a digit that isn't in its row/col/box.
        let mut none_chars: Vec<char> = p.puzzle.chars().collect();
        let mut empty_indices: Vec<usize> = none_chars
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| if c == '.' || c == '0' { Some(i) } else { None })
            .collect();

        empty_indices.shuffle(&mut rng_thread);
        let mut modified = false;

        for idx in empty_indices {
            let row = idx / 9;
            let col = idx % 9;
            
            // Collect used digits in this row, col, and 3x3 box
            let mut used = [false; 10];
            for i in 0..9 {
                // Row
                if let Some(d) = none_chars[row * 9 + i].to_digit(10) { used[d as usize] = true; }
                // Col
                if let Some(d) = none_chars[i * 9 + col].to_digit(10) { used[d as usize] = true; }
                // Box
                let r = (row / 3) * 3 + (i / 3);
                let c = (col / 3) * 3 + (i % 3);
                if let Some(d) = none_chars[r * 9 + c].to_digit(10) { used[d as usize] = true; }
            }

            // The 'solution' contains the *correct* digit for this empty cell.
            let correct_digit = p.solution.chars().nth(idx).unwrap().to_digit(10).unwrap() as usize;

            // Find an *incorrect* digit that is valid locally
            let mut possible_bad_digits = Vec::new();
            for (d, &is_used) in used.iter().enumerate().skip(1) {
                if !is_used && d != correct_digit {
                    possible_bad_digits.push(d);
                }
            }

            if !possible_bad_digits.is_empty() {
                let chosen_bad_digit = *possible_bad_digits.choose(&mut rng_thread).unwrap();
                none_chars[idx] = std::char::from_digit(chosen_bad_digit as u32, 10).unwrap();
                modified = true;
                break;
            }
        }

        if !modified {
            // Fallback: just break a known clue if we couldn't find a sneaky empty spot
            let mut clue_indices: Vec<usize> = none_chars
                .iter()
                .enumerate()
                .filter_map(|(i, &c)| if c != '.' && c != '0' { Some(i) } else { None })
                .collect();
            clue_indices.shuffle(&mut rng_thread);
            none_chars[clue_indices[0]] = if none_chars[clue_indices[0]] == '9' { '1' } else { '9' };
        }
        
        writeln!(none_file, "{}", none_chars.iter().collect::<String>()).unwrap();
    }
}
