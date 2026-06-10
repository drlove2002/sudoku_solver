# Sudoku Solver — Minigrid Permutation Graph

Solves Sudoku by enumerating valid completions of each K×K minigrid independently, building a compatibility graph between them, pruning via exact global support search, and extracting full boards. Generic over board size N×N where N is a perfect square (K = √N).

## Binaries

All binaries live in `src/bin/`. Build with `cargo build --release --bin <name>`.

### Experiment Pipeline (three-stage)

| Binary | Stage | What it does | Input | Output |
|--------|-------|-------------|-------|--------|
| `generate_solution_bank` | 1 | Solves all 16×16 and 25×25 puzzles from SAMPLE INSTANCES and writes full solved boards as seed material for dataset generation. Handles duplicate-clue panics gracefully. | `data/SAMPLE INSTANCES.txt` | `data/solutions_16x16.txt`, `data/solutions_25x25.txt`, `data/bank_manifest.txt` |
| `generate_classified_datasets` | 2 | Rejection-samples unique puzzles at six difficulty categories per board size. Strips cells from solved seeds, verifies uniqueness after each removal with `classify_up_to_two()`, stops when clue count hits the target range. 9×9 Very Easy/Easy/Medium generated from solutions; Hard/Very Hard sampled from the 3M Kaggle dataset. | Solution banks + `data/sudoku-3m.csv` | `data/classified/{size}_{category}.txt` (18 files) |
| `run_experiments` | 3 | Runs every classified puzzle through the solver twice (heuristics on → off). Checkpoint-resumes from CSV on crash. Writes per-phase breadcrumb files for OOM detection. On restart, `flush_oom_breadcrumbs()` reads orphaned breadcrumbs and marks OOM rows. | `data/classified/` | `results/experiment_results.csv` (9,112 rows × 26 cols) |

**Usage:**
```bash
# Stage 1: Solution banks (16×16 and 25×25 only — 9×9 uses the 3M dataset)
./target/release/generate_solution_bank 16
./target/release/generate_solution_bank 25

# Stage 2: Classified puzzle sets (-s size, -n count per category)
./target/release/generate_classified_datasets -s 9 -n 500
./target/release/generate_classified_datasets -s 16 -n 500
./target/release/generate_classified_datasets -s 25 -n 500

# Stage 3: Run experiments (checkpoint-resume safe — restart after OOM)
./target/release/run_experiments -- --size 9
./target/release/run_experiments -- --size 16
./target/release/run_experiments -- --size 25    # 25×25 h=off will OOM — re-run to resume
```

### Analysis & Reporting

| Script | What it does | Input | Output |
|--------|-------------|-------|--------|
| `scripts/populate_results.py` | Reads `experiment_results.csv`, computes per-category averages, writes into the `example.xlsx` template with auto-scaled memory units (B/KB/MB) and OOM annotations | `results/experiment_results.csv` + `example.xlsx` | `results/experiment_results.xlsx` |
| `scripts/generate_figures.py` | Generates 6 publication-quality PNG charts from the CSV: total time comparison, per-phase breakdown, memory usage, speedup factors, OOM analysis, and constraint propagation coverage | `results/experiment_results.csv` | `results/figures/` (6 PNGs) |

### Research & Diagnostics

| Binary | What it does | When to use |
|--------|-------------|-------------|
| `benchmark` | Benchmarks the solver over the 3M dataset in three modes: full solve stats, classify-up-to-two, and solve-one. Writes `results/benchmark_timings.csv` and `results/benchmark_timings_long.csv`. | Profiling solver performance across puzzle difficulties; measuring `classify_up_to_two()` latency |
| `profile_permutations` | Profiles Phase 2 permutation generation for 25×25. Counts solutions and branching stats per minigrid without storing permutations. | Diagnosing 25×25 permutation explosion; measuring DFS branching factors |
| `analyze_hints` | Measures how solution uniqueness degrades as clues are removed. Strips hints from solved puzzles, classifies after each removal, writes `results/degradation.csv`. | Understanding the minimum-clue threshold for unique solutions; validating ambiguity generation |
| `generate_datasets` | Stratified-samples 100 easy, 100 medium, and 100 hard puzzles from the 3M Kaggle dataset (clue-count heuristics). Writes `data/sample_easy.txt`, etc. | Quick sampling for ad-hoc testing; older version superseded by `generate_classified_datasets` |

## Solver Architecture

Six-phase pipeline in `src/solver/`:

1. **Mask init** — row/col/box conflict bitmasks from given cells
2. **Constraint propagation** — exact inference: naked/hidden singles, naked/hidden pairs, pointing/claiming pairs (togglable via `SudokuSolver::without_heuristics()`)
3. **Permutation generation** — per-minigrid DFS with MRV heuristic
4. **Graph construction** — pairwise compatibility edges between minigrid permutations
5. **Pruning** — exact global support search, removes permutations that cannot appear in any complete configuration
6. **Extraction** — backtracking configuration search + board reconstruction + validation

## Experiment Suite (June 2026)

Classified 4,556 puzzles across 9×9, 16×16, and 25×25 boards by clue-count difficulty. Ran each puzzle through the solver twice — with and without constraint propagation. Collected per-phase timing (ns), memory (B), graph vertex/edge counts, and OOM breadcrumbs.

Key result: constraint propagation delivers 50–670× speedup on 16×16 and 25×25 boards. Without it, 16 of 27 25×25 puzzles exhaust memory during graph construction.

## Tests

```bash
cargo test
cargo clippy
cargo fmt
```
