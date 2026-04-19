#!/usr/bin/env bash
set -e

echo "=========================================="
echo "Running Complete Benchmark Suite"
echo "=========================================="
echo ""

# Check if sample files exist
if [ ! -f "data/sample_easy.txt" ] || [ ! -f "data/sample_medium.txt" ] || [ ! -f "data/sample_hard.txt" ]; then
    echo "ERROR: Sample files not found in data/"
    echo "Please run: cargo run --release --bin analyze curate data/sudoku-3m.csv"
    exit 1
fi

echo "Step 1: Running microbenchmarks..."
cargo bench --bench microbenchmarks
echo "  ✓ Microbenchmarks complete"
echo ""

echo "Step 2: Running integration benchmarks..."
cargo bench --bench integration
echo "  ✓ Integration benchmarks complete"
echo ""

echo "Step 3: Running dataset analysis..."
cargo run --release --example analyze_dataset
echo "  ✓ Dataset analysis complete"
echo ""

echo "Step 4: Collecting results..."
mkdir -p results

# Copy criterion reports
if [ -d "target/criterion" ]; then
    echo "  Copying criterion reports..."
    find target/criterion -name "report" -type d | head -n 5
fi

echo ""
echo "=========================================="
echo "Benchmark Suite Complete!"
echo "=========================================="
echo ""
echo "Results location:"
echo "  - Microbenchmarks: target/criterion/*/report/index.html"
echo "  - Integration data: results/*_analysis.csv"
echo ""
echo "Next step: Generate paper figures"
echo "  python3 scripts/generate_figures.py"
