# Documentation

This directory now holds both the narrow algorithm note and the broader project paper for `sudoku_solver`.

## Papers

- `minigrid_relationship.tex`
  Focused paper on the branch-free minigrid relationship primitive used in graph construction.
- `sudoku_solver_complete.tex`
  Holistic paper covering the full solver pipeline: masks, permutation generation, graph construction, pruning, extraction, benchmarking, and analysis workflow.

## Figures

- `figures/`
  Generated plots consumed by the full paper. These are produced from CSVs in `results/`.

## Build

```bash
make -C docs all
```

This builds:

- `docs/minigrid_relationship.pdf`
- `docs/sudoku_solver_complete.pdf`

## Full Paper Pipeline

```bash
scripts/run_paper_pipeline.sh
```

This will:

1. run dataset analysis
2. generate plots into `docs/figures/`
3. build the LaTeX papers

## Cleaning

```bash
make -C docs clean
```

This removes LaTeX auxiliary files but keeps PDFs.

```bash
make -C docs cleanall
```

This removes auxiliary files and generated PDFs.
