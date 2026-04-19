# Dataset Directory

This directory contains Sudoku puzzles for benchmarking.

## Required Files

### sudoku-3m.csv (not included)

Download the Kaggle 3-million Sudoku dataset:

**Automatic download:**
```bash
../scripts/download_dataset.sh
```

**Manual download:**
1. Visit: https://www.kaggle.com/datasets/radcliffe/3-million-sudoku-puzzles-with-ratings
2. Download and extract
3. Place `sudoku-3m.csv` in this directory

### Sample Files (generated)

After downloading the dataset, generate curated samples:

```bash
cargo run --release --bin analyze curate sudoku-3m.csv
```

This creates:
- `sample_easy.txt` - 100 easy puzzles (difficulty < 0.3)
- `sample_medium.txt` - 100 medium puzzles (0.3 ≤ difficulty < 0.7)
- `sample_hard.txt` - 100 hard puzzles (difficulty ≥ 0.7)

## Format

Sample files contain one puzzle per line (81 digits, 0 = empty):

```
004300209005009001070060043006002087190007400050083000600000105003508690042910300
...
```
