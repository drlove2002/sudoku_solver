#!/usr/bin/env bash
set -e

# Define directories
DATA_DIR="$(pwd)/data"
DOWNLOAD_URL="https://www.kaggle.com/api/v1/datasets/download/radcliffe/3-million-sudoku-puzzles-with-ratings"
ZIP_FILE="$DATA_DIR/sudoku.zip"
TARGET_CSV="$DATA_DIR/sudoku-3m.csv"

# Ensure data directory exists
mkdir -p "$DATA_DIR"

if [ -f "$TARGET_CSV" ]; then
    echo "Dataset already exists at $TARGET_CSV. Skipping download."
    exit 0
fi

echo "Downloading 3-million-sudoku-puzzles-with-ratings dataset..."
curl -L -o "$ZIP_FILE" "$DOWNLOAD_URL"

echo "Extracting dataset..."
unzip -o "$ZIP_FILE" -d "$DATA_DIR"

# Rename extracted file to something predictable if it doesn't match
if [ ! -f "$TARGET_CSV" ]; then
    # The dataset usually unzips as 'sudoku-3m.csv' but just in case:
    EXTRACTED_FILE=$(ls "$DATA_DIR"/*.csv | head -n 1)
    mv "$EXTRACTED_FILE" "$TARGET_CSV"
fi

echo "Cleaning up zip file..."
rm "$ZIP_FILE"

echo "Dataset ready at $TARGET_CSV"
