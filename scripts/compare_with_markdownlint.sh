#!/usr/bin/env bash
#
# Compare mdbook-lint output with markdownlint for compatibility testing
#
# Usage: ./scripts/compare_with_markdownlint.sh [files_or_dirs...]
#
# If no arguments provided, uses test corpus files.
#
# Requirements:
#   - markdownlint-cli installed (npm install -g markdownlint-cli)
#   - mdbook-lint built (cargo build --release)
#
# Output:
#   - Summary of rule-by-rule comparison
#   - Files where tools disagree

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MDBOOK_LINT="${PROJECT_ROOT}/target/release/mdbook-lint"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
VERBOSE=false
declare -a FILES=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--verbose] [files_or_dirs...]"
            echo ""
            echo "Compare mdbook-lint and markdownlint output on markdown files."
            echo ""
            echo "Options:"
            echo "  --verbose   Show detailed per-file differences"
            echo "  --help      Show this help message"
            echo ""
            echo "If no files specified, uses test corpus files."
            exit 0
            ;;
        *)
            FILES+=("$1")
            shift
            ;;
    esac
done

# Check prerequisites
check_prerequisites() {
    if ! command -v markdownlint &> /dev/null; then
        echo -e "${RED}Error: markdownlint-cli not found${NC}"
        echo "Install with: npm install -g markdownlint-cli"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        echo -e "${RED}Error: jq not found${NC}"
        echo "Install with: brew install jq (macOS) or apt install jq (Linux)"
        exit 1
    fi

    if [[ ! -x "$MDBOOK_LINT" ]]; then
        echo -e "${YELLOW}Building mdbook-lint...${NC}"
        cd "$PROJECT_ROOT"
        cargo build --release
    fi
}

# Find markdown files
find_markdown_files() {
    local search_paths=()

    if [[ ${#FILES[@]} -eq 0 ]]; then
        # Default to test corpus
        search_paths=(
            "$PROJECT_ROOT/tests/corpus"
            "$PROJECT_ROOT/crates/mdbook-lint-cli/tests/corpus"
            "$PROJECT_ROOT/crates/mdbook-lint-cli/tests/fixtures/markdown"
        )
    else
        search_paths=("${FILES[@]}")
    fi

    for path in "${search_paths[@]}"; do
        if [[ -d "$path" ]]; then
            find "$path" -name "*.md" -type f 2>/dev/null
        elif [[ -f "$path" ]]; then
            echo "$path"
        fi
    done
}

# Count occurrences of a pattern in a file
count_in_file() {
    local pattern="$1"
    local file="$2"
    local count
    count=$(grep -c "^${pattern}$" "$file" 2>/dev/null) || count=0
    echo "$count"
}

# Main comparison logic
main() {
    check_prerequisites

    echo -e "${BLUE}=== mdbook-lint vs markdownlint Comparison ===${NC}"
    echo ""

    # Collect files into array
    local md_files=()
    while IFS= read -r file; do
        [[ -n "$file" ]] && md_files+=("$file")
    done < <(find_markdown_files)

    if [[ ${#md_files[@]} -eq 0 ]]; then
        echo -e "${YELLOW}No markdown files found to compare${NC}"
        exit 0
    fi

    echo "Comparing ${#md_files[@]} files..."
    echo ""

    # Track statistics using temp files
    local tmp_ml=$(mktemp)
    local tmp_mdl=$(mktemp)
    trap "rm -f $tmp_ml $tmp_mdl" EXIT

    local files_compared=0

    for file in "${md_files[@]}"; do
        files_compared=$((files_compared + 1))

        # Get mdbook-lint results (only MD### standard rules for fair comparison)
        "$MDBOOK_LINT" lint "$file" 2>/dev/null | \
            grep -oE '\bMD[0-9]{3}\b' >> "$tmp_ml" || true

        # Get markdownlint results (markdownlint outputs JSON to stdout even on violations)
        # but exits non-zero, so we capture output before the exit
        local mdl_output
        mdl_output=$(markdownlint --json "$file" 2>&1) || true
        echo "$mdl_output" | jq -r '.[].ruleNames[0]' >> "$tmp_mdl" 2>/dev/null || true

        if $VERBOSE; then
            echo "  $(basename "$file")"
        fi
    done

    # Count violations by rule
    echo ""
    echo -e "${BLUE}=== Summary ===${NC}"
    echo ""
    echo "Files compared: $files_compared"
    echo ""

    # Get unique rules from both outputs
    local all_rules
    all_rules=$(cat "$tmp_ml" "$tmp_mdl" 2>/dev/null | grep -E '^MD[0-9]{3}$' | sort -u || true)

    if [[ -z "$all_rules" ]]; then
        echo "No MD### violations found in either tool."
        echo ""
        echo -e "${GREEN}Comparison complete!${NC}"
        exit 0
    fi

    echo -e "${BLUE}Rule-by-Rule Comparison:${NC}"
    echo ""
    printf "%-10s %15s %15s %10s\n" "Rule" "mdbook-lint" "markdownlint" "Diff"
    printf "%-10s %15s %15s %10s\n" "----" "-----------" "------------" "----"

    local total_ml=0
    local total_mdl=0
    local matching_rules=0
    local total_rules=0

    for rule in $all_rules; do
        [[ -z "$rule" ]] && continue

        local ml_count
        local mdl_count
        ml_count=$(count_in_file "$rule" "$tmp_ml")
        mdl_count=$(count_in_file "$rule" "$tmp_mdl")

        local diff=$((ml_count - mdl_count))

        total_ml=$((total_ml + ml_count))
        total_mdl=$((total_mdl + mdl_count))
        total_rules=$((total_rules + 1))

        local diff_str=""
        if [[ $diff -gt 0 ]]; then
            diff_str="+$diff"
        elif [[ $diff -lt 0 ]]; then
            diff_str="$diff"
        else
            diff_str="="
            matching_rules=$((matching_rules + 1))
        fi

        # Color based on difference
        if [[ $diff -eq 0 ]]; then
            printf "%-10s %15d %15d ${GREEN}%10s${NC}\n" "$rule" "$ml_count" "$mdl_count" "$diff_str"
        else
            printf "%-10s %15d %15d ${YELLOW}%10s${NC}\n" "$rule" "$ml_count" "$mdl_count" "$diff_str"
        fi
    done

    echo ""
    printf "%-10s %15d %15d\n" "TOTAL" "$total_ml" "$total_mdl"

    # Calculate compatibility percentage
    if [[ $total_rules -gt 0 ]]; then
        local compat_pct=$((matching_rules * 100 / total_rules))
        echo ""
        echo -e "Rule compatibility: ${matching_rules}/${total_rules} rules match exactly (${compat_pct}%)"
    fi

    echo ""
    echo -e "${GREEN}Comparison complete!${NC}"
}

main "$@"
