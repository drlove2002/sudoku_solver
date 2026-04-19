# AGENTS.md

## Project Overview

`sudoku_solver` is a Rust research project that solves Sudoku by working at the minigrid level instead of filling the whole board cell-by-cell with a traditional global backtracker.

The core idea is:

1. Build conflict masks from the starting board.
2. Enumerate every valid completion of each 3x3 minigrid independently.
3. Connect compatible minigrid completions in a graph.
4. Search that graph for globally consistent combinations.
5. Reconstruct solved boards from those combinations.

The repository contains:

- A library crate with the solver implementation in `src/`
- A binary entrypoint in `src/main.rs`
- Integration tests in `tests/`
- Benchmarks in `benches/`
- Research notes and papers in `docs/` and `sudoku_paper.md`
- Dataset and analysis helpers in `data/`, `dataset/`, `results/`, `examples/`, and `scripts/`

This is not a generic “throw heuristics at Sudoku” codebase. The implementation is organized around minigrid permutation generation, compatibility edges, exact support search, and board reconstruction.

## Build And Run

Activate the project environment before relying on local tools:

```bash
cd /data/Projects/sudoku_solver
direnv allow
```

Common commands:

```bash
cargo build
cargo build --release
cargo run
cargo run -- dataset/simple_test.txt
cargo test
cargo test <name>
cargo bench
cargo clippy
cargo fmt
```

Operational notes:

- `src/main.rs` reads a puzzle file from `argv[1]`, defaulting to `dataset/simple_test.txt`
- Logs are written to `trace.log` and standard output
- Graph export is wired in the codebase but currently commented out in `SudokuSolver::solve_with_stats`

## Repository Layout

- `src/lib.rs`: library entrypoint and public exports
- `src/main.rs`: binary entrypoint, file input, logger setup, top-level orchestration
- `src/solver/mod.rs`: `SudokuSolver`, `SolveReport`, `SolveStats`, phase orchestration
- `src/solver/permutations.rs`: minigrid-local DFS permutation generation
- `src/solver/pruning.rs`: exact support pruning over the compatibility graph
- `src/solver/extraction.rs`: configuration search and solution reconstruction
- `src/types/board.rs`: board storage, formatting, validity checking
- `src/types/minigrid.rs`: flattened minigrid view over the board
- `src/types/masks/`: bitmask primitives and conflict mask generation
- `src/types/graph/`: graph container, permutation nodes, relationships, compatibility logic
- `src/dataset_parser.rs`: CSV parsing and Kaggle puzzle-string parsing
- `tests/integration_tests.rs`: end-to-end solver checks against sample puzzles
- `benches/benchmark.rs`: microbenchmarks for graph relationship logic

Notes:

- `src/solver/graph.rs` is currently a placeholder and is not part of the active pipeline.
- `src/types/logic/mod.rs` is currently empty.

## High-Level Architecture

The main solver type is `SudokuSolver<const N: usize, const K: usize>` in `src/solver/mod.rs`.

For the standard puzzle configuration used here:

- `N = 9`
- `K = 3`

The solver works in five explicit phases inside `SudokuSolver::solve_with_stats()`:

1. Generate row, column, box, and per-cell conflict masks with `Masks::<N>::generate`.
2. Generate all valid minigrid completions with `SudokuSolver::generate_all_permutations`.
3. Build the compatibility graph with `Graph::new` and `Graph::create_edges`.
4. Remove unsupported permutations with `prune_graph_iterative`.
5. Extract full board solutions with `extract_solutions`.

The important architectural property is that the implementation separates:

- Local reasoning inside a minigrid
- Pairwise compatibility between minigrids
- Global consistency search across all minigrids
- Reconstruction of final boards

That separation is intentional. Preserve it unless there is a strong reason to change it.

## Core Concepts

### `Board<N>`

Defined in `src/types/board.rs`.

- Stores the puzzle as `[[u8; N]; N]`
- Computes minigrid IDs with `Board::box_idx`
- Validates finished or reconstructed boards with `Board::is_valid`
- Assumes `N` is a perfect square via `Board::new`

`Board` is the canonical whole-puzzle representation. If a change affects puzzle semantics, start by checking whether `Board` invariants still hold.

### `Masks<N>`

Defined in `src/types/masks/mod.rs`.

- `rows`, `cols`, `boxs`: digits already used in each row, column, and minigrid
- `conflict[r][c]`: combined forbidden-digit mask for a specific cell

`Masks::generate` is phase 1. It rejects invalid input boards by panicking on duplicate values in a row, column, or box.

### `BitString<N>`, `DirtyMask<N>`, `EmptyMask<N>`

Defined in `src/types/masks/bitstring.rs`.

- `BitString<N>` is the basic `u32`-backed bitmask
- `DirtyMask<N>` represents used digits
- `EmptyMask<N>` represents empty local positions in a minigrid

Key conventions:

- `DirtyMask` is 1-based at the API boundary: `dirty_set(1)` sets the bit for digit `1`
- `EmptyMask` iterates set bits using Kernighan’s trick
- This implementation assumes `N` fits within the `u32` backing mask

### `Minigrid<N, K>`

Defined in `src/types/minigrid.rs`.

- Represents one KxK box as a flattened `[u8; N]`
- Stores its minigrid ID in row-major order
- Stores empty local positions in `empty: EmptyMask<N>`

`Minigrid::new` builds a minigrid view from the full board. The minigrid is the working unit for phase 2.

### `PermutationNode<N, K>`

Defined in `src/types/graph/node.rs`.

- Represents one valid full filling of a minigrid
- Stores the minigrid cell values in `cells`
- Precomputes `row_masks` and `col_masks` for the minigrid-local rows and columns
- Stores graph edges in `compatible: Vec<(usize, usize)>`

This is the graph vertex type. A `PermutationNode` is not a partial assignment. It is a complete, valid filling for one minigrid.

### `Graph<K, N>`

Defined in `src/types/graph/mod.rs`.

- Stores `[Vec<PermutationNode<N, K>>; N]`, one vector per minigrid
- Builds pairwise compatibility edges with `Graph::create_edges`
- Supports exact pruning through `retain_permutations`
- Exposes read access used by extraction and validation

The graph is partitioned by minigrid. Each minigrid owns its candidate permutation set.

### `Relation`

Defined in `src/types/graph/relationship.rs`.

- `Relation::Row`: two minigrids share a block row
- `Relation::Col`: two minigrids share a block column
- `Relation::Not`: no direct row/column compatibility constraint between them

`Graph::relationship(a, b)` determines how two minigrids can constrain each other.

## How The Main Components Interact

The interaction order is:

1. `Board` feeds `Masks`.
2. `Masks` and `Board` feed `Minigrid` permutation generation.
3. Each generated minigrid completion becomes a `PermutationNode`.
4. All `PermutationNode`s are grouped into a `Graph`.
5. The graph adds compatibility edges based on `Relation` plus row/column mask overlap checks.
6. Pruning runs a global configuration search and keeps only supported nodes.
7. Extraction reruns the configuration search on the pruned graph and reconstructs full `Board`s.

Two points matter here:

- Local generation and global search are different stages.
- Pruning and extraction intentionally both reason about full graph configurations.

That second point can look redundant at first glance, but it keeps concerns separate:

- pruning answers “which permutations can appear in at least one valid board?”
- extraction answers “which full boards exist, and what do they look like?”

## Solver Pipeline

### Phase 1: Parsing And Mask Initialization

Entry points:

- `src/main.rs`
- `Masks::<N>::generate`

Behavior:

- `main` reads whitespace-separated digits from a file into `Board<9>`
- `Masks::generate` computes row, column, box, and per-cell conflict masks
- Duplicate givens cause an immediate panic

Output of this phase:

- A validated input board
- Fast conflict lookups for each cell

### Phase 2: Minigrid Permutation Generation

Entry points:

- `SudokuSolver::generate_all_permutations`
- `Minigrid::generate_permutations_dfs`
- `Minigrid::find_best_cell`

Behavior:

- Each minigrid is processed independently in parallel with Rayon
- The search fills only cells inside that minigrid
- Existing digits in the minigrid seed the `used_mask`
- `find_best_cell` applies an MRV-style heuristic by selecting the empty cell with the most constrained candidate set
- Each complete local filling becomes a `PermutationNode`

Important constraint:

- Phase 2 respects row, column, and box conflicts from the original board plus digits already placed within the same minigrid
- It does not yet enforce consistency with other minigrids beyond what the initial masks already imply

### Phase 3: Graph Construction

Entry points:

- `Graph::new`
- `Graph::create_edges`
- `PermutationNode::check_row_compatible`
- `PermutationNode::check_col_compatible`

Behavior:

- The graph compares each minigrid pair `(i, j)`
- `Graph::relationship(i, j)` classifies whether the pair shares row constraints, column constraints, or neither
- For related minigrid pairs, each permutation pair is tested for mask overlap
- Compatible pairs receive symmetric edges in `compatible`

Result:

- The graph encodes all pairwise compatible local completions

### Phase 4: Pruning

Entry point:

- `prune_graph_iterative`

What it actually does:

- Despite the name, this is not a local iterative degree-pruning pass
- It performs an exact global support search by calling `find_all_configurations`
- Every permutation that appears in at least one full valid configuration is marked as supported
- `Graph::retain_permutations` rebuilds the graph with only supported nodes and remapped edges

This means “pruning” in the current implementation is better understood as:

- exact support search
- support-based graph reduction

If you keep the current function name, document that distinction when making related changes.

### Phase 5: Solution Extraction

Entry points:

- `extract_solutions`
- `find_all_configurations`
- `reconstruct_board`

Behavior:

- Finds all complete minigrid-permutation assignments consistent with the graph
- Reconstructs full boards from those assignments
- Validates reconstructed boards with `Board::is_valid`
- Classifies the puzzle as `Unsolvable`, `Unique`, or `Ambiguous(n)`

The extraction search uses:

- backtracking
- MRV-style minigrid selection via `select_next_minigrid`
- pairwise edge checks via `is_compatible`

## Notes On Pruning And Solution Extraction

The most important implementation detail in this repository is that pruning is already an exact global search.

That has several consequences:

- Pruning is correct in the strong sense that unsupported permutations are removed only if they cannot appear in any full solution.
- Hard puzzles can still be expensive because the exact search happens before extraction.
- Extraction currently repeats the same configuration search after pruning.

This duplication is acceptable if the goals are clarity and stage separation. It becomes a performance issue only if measurements show it matters.

If you optimize this area:

- Preserve correctness first.
- Be explicit about whether you are changing semantics or only avoiding repeated work.
- Keep the distinction between “supported permutation” and “reconstructed solution board” clear.

## Coding Style Guidelines

The preferred style in this repository is simple, explicit, and easy to reason about.

Write code that:

- Keeps control flow visible
- Keeps data ownership obvious
- Uses small, narrow functions for each phase
- Uses types and constructors to encode invariants where practical
- Separates algorithm stages instead of blending them together

Prefer:

- Straightforward structs over clever abstraction layers
- Explicit loops when they make the algorithm easier to inspect
- Names that match the paper and the current implementation
- Pure helper logic in the library crate
- `main.rs` only for input, logging, and orchestration

Avoid:

- “Framework-like” refactors that hide the solver pipeline
- Pushing core solver logic into the binary
- Introducing generic abstractions that make the graph or minigrid model harder to follow
- Premature micro-optimizations without benchmark evidence

## Design Principles

### Simplicity Over Cleverness

The code should be understandable from top to bottom by reading the solver phases in order. If a refactor makes the code shorter but less obvious, it is usually the wrong trade.

### Modularity By Solver Stage

Each stage should have a clear contract:

- masks describe fixed conflicts from the input board
- permutations describe valid local minigrid completions
- graph edges describe pairwise compatibility
- pruning describes global support
- extraction describes board reconstruction

Do not collapse these into one opaque solving routine unless there is a compelling reason.

### Explicit Control Flow

This project favors visible search steps and direct data movement. Hidden mutation, deeply indirect callbacks, or over-composed iterator pipelines are usually a poor fit for the core solver.

### Correctness First, Then Performance

Performance matters, but only after correctness and inspectability are preserved. The repository already contains benchmarks; use them before claiming an optimization helps.

## Do / Don’t Guidelines

Do:

- Keep core algorithm changes in `src/solver/` and `src/types/`
- Preserve `Board`, `Minigrid`, `PermutationNode`, and `Graph` as the main mental model
- Add tests when changing solver behavior
- Update docs when implementation semantics change
- Keep graph edges symmetric
- Keep pruning semantics explicit if you rename or refactor it
- Re-run `cargo test`, and usually `cargo clippy` and `cargo fmt`, after meaningful Rust changes

Don’t:

- Don’t invent new architecture terms that do not exist in the code
- Don’t describe pruning as a cheap local elimination pass unless you actually change the implementation
- Don’t bypass `Board::is_valid` checks when reconstructing new solution paths
- Don’t weaken the bitmask conventions without updating all call sites
- Don’t mix dataset-processing code into the core solver
- Don’t make performance claims without `cargo bench` or equivalent measurement
- Don’t remove stage boundaries just to reduce file count

## Key Invariants And Assumptions

- `N` must be a perfect square
- The current bitmask implementation assumes `N <= 32`
- Digits use `1..=N`; `0` means empty
- A `PermutationNode` represents a complete minigrid assignment, not a partial one
- Compatibility edges are only meaningful for minigrids related by shared block row or block column
- `Graph::retain_permutations` must remap edge indices consistently after pruning
- Reconstructed boards must still pass `Board::is_valid`

If you change any of these assumptions, update the relevant types, tests, and documentation together.

## How To Modify The Solver Safely

When changing the algorithm:

1. Identify which phase owns the behavior you want to change.
2. Change that phase in isolation first.
3. Verify the inputs and outputs of adjacent phases still match.
4. Add or update tests at the phase boundary you touched.
5. Run end-to-end solver tests after the local change works.

Examples:

- If you change candidate generation, start in `src/solver/permutations.rs` and re-check graph sizes and solution counts.
- If you change compatibility semantics, start in `src/types/graph/relationship.rs` or `src/types/graph/compatibility.rs` and verify pruning/extraction still agree.
- If you change support search, update both `src/solver/pruning.rs` and `src/solver/extraction.rs` if the configuration search contract changes.
- If you change board reconstruction, verify `Board::is_valid` still passes for extracted solutions.

## How To Extend The Codebase

Safe extensions usually fit into one of these categories:

### Add Instrumentation Or Reporting

Preferred locations:

- `SolveStats` in `src/solver/mod.rs`
- logging in the existing phase boundaries
- analysis helpers outside the solver core

### Add New Search Heuristics

Preferred locations:

- `find_best_cell` for local minigrid generation
- `select_next_minigrid` for global configuration search

When doing this:

- Keep the heuristic separate from correctness logic
- Preserve the exact meaning of a valid configuration

### Add New Validation Or Debug Views

Preferred locations:

- `Board::is_valid`
- graph helper methods
- dedicated debug or visualization modules

### Add New Puzzle Input Formats

Preferred locations:

- `src/main.rs` for CLI/file handling
- `src/dataset_parser.rs` for reusable parsing helpers

Do not entangle input parsing with solver internals.

## Testing And Benchmarking Expectations

Use:

- `cargo test` for correctness
- `cargo test <name>` for targeted work
- `cargo bench` for performance-sensitive changes
- `cargo clippy` for linting
- `cargo fmt` for formatting

Current coverage focuses on:

- sample puzzle solving in `tests/integration_tests.rs`
- exact pruning behavior in `src/solver/pruning.rs` tests
- relationship logic in `src/types/graph/relationship.rs` tests
- microbenchmarks for relationship computation in `benches/benchmark.rs`

If you change algorithm semantics, prefer adding tests close to the affected phase and at least one end-to-end solver check.

## Final Guidance

Treat the library crate as the canonical implementation of the algorithm.

Keep the code aligned with the actual pipeline:

1. masks
2. minigrid permutations
3. graph construction
4. exact support pruning
5. solution extraction

When in doubt, choose the design that makes those five steps easier to read, test, and trust.
