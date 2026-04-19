# Results Directory

This directory contains reproducible analysis outputs used by the paper and future benchmarking scripts.

## Dataset Analysis Outputs

After running:

```bash
cargo run --release --example analyze_dataset
```

the solver writes:

- `easy_analysis.csv`
- `medium_analysis.csv`
- `hard_analysis.csv`
- `complete_analysis.csv`
- `phase_breakdown.csv`
- `statistics_summary.txt`

These files capture:

- phase 2 permutation counts
- phase 3 graph sizes and edge-build timing
- phase 4 pruning reductions and timing
- phase 5 solution counts and classification
- end-to-end phase timing totals

## Figure Pipeline

`scripts/generate_figures.py` consumes these CSVs and writes plots to:

- `docs/figures/`

## Benchmark Artifacts

Criterion benchmark HTML remains under:

- `target/criterion/*/report/index.html`

Those reports are not copied into this directory by default.
