#!/bin/bash

# Performance benchmark script for mdbook-lint

set -e

LINTER="./target/release/mdbook-lint"
TEST_DIR="/tmp/rust-book"

echo "=== mdbook-lint Performance Benchmark ==="
echo ""

# Build release binary if needed
if [ ! -f "$LINTER" ]; then
    echo "Building release binary..."
    cargo build --release
fi

# Clone test repo if needed
if [ ! -d "$TEST_DIR" ]; then
    echo "Cloning The Rust Book..."
    git clone --depth 1 https://github.com/rust-lang/book.git "$TEST_DIR"
fi

echo ""
echo "Test corpus: The Rust Book"
echo "Location: $TEST_DIR"

# Count files
MD_COUNT=$(find "$TEST_DIR" -name "*.md" | wc -l | tr -d ' ')
echo "Markdown files: $MD_COUNT"

# Get total size
TOTAL_SIZE=$(find "$TEST_DIR" -name "*.md" -exec wc -c {} + | tail -1 | awk '{print $1}')
echo "Total size: $(echo "scale=2; $TOTAL_SIZE / 1024 / 1024" | bc) MB"

echo ""
echo "Running performance tests..."
echo ""

# Test 1: Single chapter
echo "Test 1: Single chapter (ch01-*.md)"
time $LINTER lint "$TEST_DIR"/src/ch01-*.md --quiet 2>&1 | tail -1

echo ""

# Test 2: First 10 chapters  
echo "Test 2: First 10 chapters (ch0*.md, ch1*.md)"
time $LINTER lint "$TEST_DIR"/src/ch0*.md "$TEST_DIR"/src/ch1*.md --quiet 2>&1 | tail -1

echo ""

# Test 3: All src files with timeout
echo "Test 3: All src files (with 30s timeout)"
timeout 30 time $LINTER lint "$TEST_DIR"/src/*.md --quiet 2>&1 | tail -1 || echo "Timeout after 30 seconds"

echo ""

# Test 4: Specific rules only
echo "Test 4: All files with only MD rules (no MDBOOK rules)"
time $LINTER lint "$TEST_DIR"/src/*.md --disable MDBOOK001,MDBOOK002,MDBOOK003,MDBOOK004,MDBOOK005,MDBOOK006,MDBOOK007,MDBOOK008,MDBOOK009,MDBOOK010,MDBOOK011,MDBOOK012,MDBOOK025 --quiet 2>&1 | tail -1

echo ""
echo "=== Benchmark Complete ==="