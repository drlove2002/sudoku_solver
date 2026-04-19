./scripts/download_dataset.sh
cargo run --release --bin generate_datasets
cargo run --release --bin benchmark
nix develop -c python3 scripts/visualize_benchmarks.py
