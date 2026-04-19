use csv::Reader;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Deserialize)]
pub struct SudokuPuzzle {
    #[serde(default)]
    pub id: usize,
    pub puzzle: String,
    pub solution: String,
    #[serde(default)]
    pub clues: usize,
    pub difficulty: f64,
}

pub fn parse_csv(path: &str) -> Result<Vec<SudokuPuzzle>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut rdr = Reader::from_reader(reader);

    let puzzles: Vec<SudokuPuzzle> = rdr.deserialize().filter_map(|result| result.ok()).collect();

    Ok(puzzles)
}

pub fn stratified_sample(
    puzzles: &[SudokuPuzzle],
    n_easy: usize,
    n_medium: usize,
    n_hard: usize,
) -> Vec<SudokuPuzzle> {
    // Based on Kaggle dataset distribution:
    // Easy: difficulty < 1.0 (1.3M puzzles)
    // Medium: 1.0 <= difficulty < 2.5 (1.1M puzzles)
    // Hard: difficulty >= 2.5 (562K puzzles)

    let mut easy: Vec<_> = puzzles
        .iter()
        .filter(|p| p.difficulty < 1.0)
        .take(n_easy)
        .cloned()
        .collect();

    let mut medium: Vec<_> = puzzles
        .iter()
        .filter(|p| p.difficulty >= 1.0 && p.difficulty < 2.5)
        .take(n_medium)
        .cloned()
        .collect();

    let mut hard: Vec<_> = puzzles
        .iter()
        .filter(|p| p.difficulty >= 2.5)
        .take(n_hard)
        .cloned()
        .collect();

    easy.append(&mut medium);
    easy.append(&mut hard);
    easy
}

pub fn parse_puzzle_string(puzzle_str: &str) -> Result<Vec<u8>, String> {
    if puzzle_str.len() != 81 {
        return Err(format!(
            "Invalid puzzle length: expected 81, got {}",
            puzzle_str.len()
        ));
    }

    puzzle_str
        .chars()
        .map(|c| {
            if c == '.' {
                Ok(0) // Kaggle format uses '.' for empty cells
            } else {
                c.to_digit(10)
                    .map(|d| d as u8)
                    .ok_or_else(|| format!("Invalid character: {}", c))
            }
        })
        .collect()
}
