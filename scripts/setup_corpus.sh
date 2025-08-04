#!/bin/bash
set -euo pipefail

# Setup script for mdbook-lint corpus testing
# This script sets up test corpora for compatibility and performance testing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CORPUS_DIR="$PROJECT_ROOT/tests/corpus"

echo "Setting up mdbook-lint corpus testing..."
echo "Project root: $PROJECT_ROOT"
echo "Corpus directory: $CORPUS_DIR"

# Create corpus directory structure
mkdir -p "$CORPUS_DIR"
mkdir -p "$CORPUS_DIR/markdownlint"
mkdir -p "$CORPUS_DIR/edge_cases"
mkdir -p "$CORPUS_DIR/real_projects"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to clone or update a repository
clone_or_update() {
    local repo_url="$1"
    local target_dir="$2"
    local depth="${3:-1}"

    if [ -d "$target_dir/.git" ]; then
        echo "Updating existing repository: $target_dir"
        cd "$target_dir"
        git pull origin main || git pull origin master || echo "Warning: Could not update $target_dir"
        cd - > /dev/null
    else
        echo "Cloning $repo_url to $target_dir"
        git clone --depth "$depth" "$repo_url" "$target_dir"
    fi
}

# Setup markdownlint official test suite
echo ""
echo "Setting up markdownlint official test suite..."
if command_exists git; then
    MARKDOWNLINT_DIR="$CORPUS_DIR/markdownlint"
    clone_or_update "https://github.com/DavidAnson/markdownlint.git" "$MARKDOWNLINT_DIR"

    # Verify we got the test files
    if [ -d "$MARKDOWNLINT_DIR/test" ]; then
        test_file_count=$(find "$MARKDOWNLINT_DIR/test" -name "*.md" | wc -l)
        echo "Found $test_file_count test files in markdownlint test suite"
    else
        echo "Warning: markdownlint test directory not found"
    fi
else
    echo "Git not available, skipping markdownlint test suite setup"
fi

# Setup real-world project samples
echo ""
echo "Setting up real-world project samples..."
REAL_PROJECTS_DIR="$CORPUS_DIR/real_projects"

# mdBook project (self-referential but good test case)
if command_exists git; then
    clone_or_update "https://github.com/rust-lang/mdBook.git" "$REAL_PROJECTS_DIR/mdbook" 2

    # Rust Book
    clone_or_update "https://github.com/rust-lang/book.git" "$REAL_PROJECTS_DIR/rust-book" 2

    # Count markdown files in real projects
    if [ -d "$REAL_PROJECTS_DIR" ]; then
        real_md_count=$(find "$REAL_PROJECTS_DIR" -name "*.md" | wc -l)
        echo "Found $real_md_count markdown files in real projects"
    fi
else
    echo "Git not available, skipping real project setup"
fi

# Generate edge cases using our built-in generator
echo ""
echo "Generating edge case test files..."
cd "$PROJECT_ROOT"

# Try to generate edge cases using our corpus test
if command_exists cargo; then
    echo "Running edge case generation..."
    cargo test test_corpus_edge_cases --release -- --nocapture || {
        echo "Note: Edge case generation may have failed, but that's okay for setup"
    }

    if [ -d "$CORPUS_DIR/edge_cases" ]; then
        edge_case_count=$(find "$CORPUS_DIR/edge_cases" -name "*.md" | wc -l)
        echo "Generated $edge_case_count edge case files"
    fi
else
    echo "Cargo not available, skipping edge case generation"
fi

# Create a summary report
echo ""
echo "Corpus setup summary:"
echo "===================="

total_files=0

if [ -d "$CORPUS_DIR/markdownlint" ]; then
    ml_count=$(find "$CORPUS_DIR/markdownlint" -name "*.md" | wc -l)
    echo "- Markdownlint test suite: $ml_count files"
    total_files=$((total_files + ml_count))
fi

if [ -d "$CORPUS_DIR/real_projects" ]; then
    rp_count=$(find "$CORPUS_DIR/real_projects" -name "*.md" | wc -l)
    echo "- Real projects: $rp_count files"
    total_files=$((total_files + rp_count))
fi

if [ -d "$CORPUS_DIR/edge_cases" ]; then
    ec_count=$(find "$CORPUS_DIR/edge_cases" -name "*.md" | wc -l)
    echo "- Edge cases: $ec_count files"
    total_files=$((total_files + ec_count))
fi

echo "- Total corpus files: $total_files"

# Check for markdownlint CLI availability
echo ""
echo "External tool availability:"
echo "=========================="

if command_exists markdownlint; then
    ml_version=$(markdownlint --version 2>/dev/null || echo "unknown")
    echo "✓ markdownlint CLI available (version: $ml_version)"
elif command_exists npx && npx markdownlint --version >/dev/null 2>&1; then
    ml_version=$(npx markdownlint --version 2>/dev/null || echo "unknown")
    echo "✓ markdownlint CLI available via npx (version: $ml_version)"
else
    echo "⚠ markdownlint CLI not available"
    echo "  Install with: npm install -g markdownlint-cli"
    echo "  Corpus tests will still work but won't include compatibility comparisons"
fi

if command_exists node; then
    node_version=$(node --version)
    echo "✓ Node.js available ($node_version)"
else
    echo "⚠ Node.js not available"
fi

echo ""
echo "Corpus setup complete!"
echo ""
echo "To run corpus tests:"
echo "  cargo test corpus_integration_test"
echo ""
echo "To run corpus tests with markdownlint comparison:"
echo "  cargo test corpus_integration_test -- --ignored"
echo ""
echo "To run performance benchmarks:"
echo "  cargo test test_performance_benchmark --release"
