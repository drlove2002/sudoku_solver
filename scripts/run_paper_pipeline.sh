#!/usr/bin/env bash
set -euo pipefail

echo "== sudoku_solver paper pipeline =="

mkdir -p results docs/figures

echo
echo "1. Running dataset analysis"
cargo run --release --example analyze_dataset -- "$@"

echo
echo "2. Generating figures"
python3 scripts/generate_figures.py

echo
echo "3. Building papers"
make -C docs all

echo
echo "Pipeline complete."
echo "Paper outputs:"
echo "  docs/minigrid_relationship.pdf"
echo "  docs/sudoku_solver_complete.pdf"
echo "Figure outputs:"
echo "  docs/figures/*.pdf"
