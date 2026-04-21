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
    let len = puzzle_str.len();
    if len != 81 && len != 256 && len != 625 {
        return Err(format!(
            "Invalid puzzle length: expected 81, 256, or 625, got {}",
            len
        ));
    }

    let is_9x9 = len == 81;

    puzzle_str
        .chars()
        .map(|c| {
            if c == '.' || c == '0' {
                Ok(0) // Format uses '.' or '0' for empty cells
            } else if is_9x9 {
                c.to_digit(10)
                    .map(|d| d as u8)
                    .ok_or_else(|| format!("Invalid character for 9x9: {}", c))
            } else {
                if c.is_ascii_alphabetic() {
                    let val = c.to_ascii_uppercase() as u8 - b'A' + 1;
                    let max_val = if len == 256 { 16 } else { 25 };
                    if val <= max_val {
                        Ok(val)
                    } else {
                        Err(format!("Character '{}' out of range for {}x{}", c, max_val, max_val))
                    }
                } else {
                    Err(format!("Invalid character for >9x9 (expected alphabet): {}", c))
                }
            }
        })
        .collect()
}
