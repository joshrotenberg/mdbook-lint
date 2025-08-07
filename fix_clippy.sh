#!/bin/bash
# Temporary script to help fix collapsible if patterns

echo "Fixing collapsible if patterns..."

# Get all files with clippy warnings
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | \
    grep -A 20 "this \`if\` statement can be collapsed" | \
    grep "crates/mdbook-lint-core/src/" | \
    sed 's/.*--> //' | \
    sed 's/:.*$//' | \
    sort | uniq

echo "Found files with collapsible if patterns above"