#!/bin/bash
set -e

NAME="minigrid-relationship-paper"
VERSION="1.0.0"
ARCHIVE="${NAME}-${VERSION}.zip"

echo "=========================================="
echo "Creating Distribution Package"
echo "=========================================="
echo ""

# Ensure everything is built
echo "Step 1: Building release binaries..."
cargo build --release
echo "  ✓ Release build complete"
echo ""

echo "Step 2: Preparing staging directory..."
STAGE="dist/${NAME}"
rm -rf dist
mkdir -p "$STAGE"

# Copy essential files
echo "  Copying source files..."
cp -r src benches examples scripts data results paper Cargo.toml README.md Makefile "$STAGE/"

# Create bin directory and copy binaries
echo "  Copying binaries..."
mkdir -p "$STAGE/bin"
if [ -f "target/release/analyze" ]; then
    cp target/release/analyze "$STAGE/bin/analyze-linux"
    echo "    ✓ Linux binary"
elif [ -f "target/release/analyze.exe" ]; then
    cp target/release/analyze.exe "$STAGE/bin/"
    echo "    ✓ Windows binary"
fi

# Include compiled PDF if it exists
if [ -f "paper/minigrid_relationship.pdf" ]; then
    cp paper/minigrid_relationship.pdf "$STAGE/"
    echo "  ✓ Including compiled PDF"
fi

# Clean up build artifacts
echo "  Cleaning build artifacts..."
find "$STAGE" -name "*.aux" -o -name "*.log" -o -name "*.out" -o -name "*.bbl" -o -name "*.blg" -delete
rm -rf "$STAGE/target"

# Create LICENSE if it doesn't exist
if [ ! -f "$STAGE/LICENSE" ]; then
    echo "  Creating MIT LICENSE..."
    cat > "$STAGE/LICENSE" << 'EOF'
MIT License

Copyright (c) 2025 Minigrid Relationship Research

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
fi

echo ""
echo "Step 3: Creating archive..."
cd dist
zip -r -q "../${ARCHIVE}" "${NAME}"
cd ..

# Generate checksums
echo "  Generating checksums..."
sha256sum "${ARCHIVE}" > "${ARCHIVE}.sha256"

echo ""
echo "=========================================="
echo "Package Complete!"
echo "=========================================="
echo ""
echo "Archive: ${ARCHIVE}"
ls -lh "${ARCHIVE}"
echo ""
echo "Checksum: ${ARCHIVE}.sha256"
cat "${ARCHIVE}.sha256"
echo ""
echo "Contents:"
unzip -l "${ARCHIVE}" | head -n 20
echo "  ..."
