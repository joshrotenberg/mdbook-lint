#!/bin/bash
# Download extended corpus for comprehensive testing
# Used in CI and for local comprehensive testing

set -euo pipefail

CORPUS_DIR="tests/corpus"
EXTENDED_CORPUS="${CORPUS_DIR}/extended"

echo "📁 Setting up extended corpus for comprehensive testing..."

# Create extended directory
mkdir -p "${EXTENDED_CORPUS}"

# Function to download and extract a GitHub repository
download_repo() {
    local repo="$1"
    local branch="${2:-main}"
    local target_dir="$3"
    
    echo "⬇️  Downloading ${repo} (${branch})..."
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    
    # Download repository
    curl -fsSL "https://github.com/${repo}/archive/${branch}.tar.gz" | \
        tar -xz -C "${temp_dir}" --strip-components=1
    
    # Move to target directory
    mkdir -p "${target_dir}"
    find "${temp_dir}" -name "*.md" -exec cp {} "${target_dir}/" \;
    
    # Cleanup
    rm -rf "${temp_dir}"
    
    local file_count
    file_count=$(find "${target_dir}" -name "*.md" | wc -l)
    echo "✅ Downloaded ${file_count} markdown files from ${repo}"
}

# Download markdownlint official test suite (if not in CI cache)
if [[ ! -d "${EXTENDED_CORPUS}/markdownlint-official" ]]; then
    download_repo "DavidAnson/markdownlint" "main" "${EXTENDED_CORPUS}/markdownlint-official"
fi

# Download popular mdBook projects for real-world testing
if [[ ! -d "${EXTENDED_CORPUS}/mdbook-official" ]]; then
    download_repo "rust-lang/mdBook" "master" "${EXTENDED_CORPUS}/mdbook-official"
fi

if [[ ! -d "${EXTENDED_CORPUS}/rust-book-latest" ]]; then
    download_repo "rust-lang/book" "main" "${EXTENDED_CORPUS}/rust-book-latest"
fi

if [[ ! -d "${EXTENDED_CORPUS}/rust-reference" ]]; then
    download_repo "rust-lang/reference" "master" "${EXTENDED_CORPUS}/rust-reference"
fi

# Download additional test projects
if [[ ! -d "${EXTENDED_CORPUS}/cargo-book" ]]; then
    download_repo "rust-lang/cargo" "master" "${EXTENDED_CORPUS}/cargo-book"
fi

# Add current project files for self-testing
echo "📄 Adding current project files for self-testing..."
mkdir -p "${EXTENDED_CORPUS}/self_project"

# Copy current project documentation
for file in README.md CLAUDE.md CONVENTIONS.md; do
    if [[ -f "$file" ]]; then
        cp "$file" "${EXTENDED_CORPUS}/self_project/"
        echo "   ✅ $file"
    fi
done

# Copy docs directory
if [[ -d "docs" ]]; then
    find docs -name "*.md" -exec cp {} "${EXTENDED_CORPUS}/self_project/" \;
    doc_count=$(find docs -name "*.md" | wc -l)
    echo "   ✅ ${doc_count} documentation files"
fi

# Generate comprehensive edge cases
echo "🔧 Generating comprehensive edge cases..."
if [[ -f "target/debug/mdbook-lint" ]] || command -v mdbook-lint >/dev/null 2>&1; then
    # If binary exists, we can generate more sophisticated test cases
    # This would be expanded with actual edge case generation
    mkdir -p "${EXTENDED_CORPUS}/generated"
    echo "# Generated Test Case" > "${EXTENDED_CORPUS}/generated/sample.md"
    echo "Generated basic test cases"
else
    echo "⚠️  mdbook-lint binary not found, skipping advanced generation"
fi

# Summary
total_files=$(find "${EXTENDED_CORPUS}" -name "*.md" | wc -l)
total_size=$(du -sh "${EXTENDED_CORPUS}" | cut -f1)

echo ""
echo "📊 Extended Corpus Summary:"
echo "   Files: ${total_files} markdown files"
echo "   Size: ${total_size}"
echo "   Location: ${EXTENDED_CORPUS}"
echo ""
echo "🎯 Use 'cargo test --test corpus_integration_test --ignored' for comprehensive testing"