#!/bin/bash
set -euo pipefail

# GitHub Action execution script for mdbook-lint
# Runs mdbook-lint with the provided inputs and handles outputs

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    if [[ "${RUNNER_DEBUG:-}" == "1" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Parse inputs with defaults
FILES="${INPUT_FILES:-**/*.md}"
CONFIG_FILE="${INPUT_CONFIG_FILE:-.mdbook-lint.toml}"
FAIL_ON_WARNINGS="${INPUT_FAIL_ON_WARNINGS:-true}"
RULES="${INPUT_RULES:-}"
FORMAT="${INPUT_FORMAT:-human}"
OUTPUT_FILE="${INPUT_OUTPUT_FILE:-}"

# Output tracking
VIOLATIONS_COUNT=0
WARNINGS_COUNT=0
ERRORS_COUNT=0
TEMP_OUTPUT=""
SARIF_FILE=""

# Create temp file for capturing output
setup_temp_output() {
    TEMP_OUTPUT=$(mktemp)
    trap cleanup EXIT
}

cleanup() {
    if [[ -n "$TEMP_OUTPUT" && -f "$TEMP_OUTPUT" ]]; then
        rm -f "$TEMP_OUTPUT"
    fi
}

# Build mdbook-lint command
build_command() {
    local cmd=("mdbook-lint" "lint")
    
    # Add configuration file if it exists
    if [[ -f "$CONFIG_FILE" ]]; then
        cmd+=("--config" "$CONFIG_FILE")
        log_info "Using config file: $CONFIG_FILE"
    else
        log_debug "Config file not found: $CONFIG_FILE"
    fi
    
    # Add format (check if CLI supports the requested format)
    if mdbook-lint lint --help 2>/dev/null | grep -q -- "--output"; then
        case "$FORMAT" in
            human)
                cmd+=("--output" "default")
                log_debug "Output format: human -> default"
                ;;
            json)
                cmd+=("--output" "json")
                log_debug "Output format: json"
                ;;
            sarif)
                # Check if sarif is supported
                if mdbook-lint lint --help 2>/dev/null | grep -q "sarif"; then
                    cmd+=("--output" "sarif")
                    log_debug "Output format: sarif"
                else
                    log_warn "SARIF format not supported in this version, falling back to json"
                    cmd+=("--output" "json")
                fi
                ;;
            *)
                log_error "Invalid format: $FORMAT (must be human, json, or sarif)"
                exit 1
                ;;
        esac
    else
        log_warn "CLI version doesn't support --output flag, using defaults"
        if [[ "$FORMAT" != "human" ]]; then
            log_warn "Requested format '$FORMAT' not supported in this version"
        fi
    fi
    
    # Add output file if specified or set default for SARIF (check if supported)
    if mdbook-lint lint --help 2>/dev/null | grep -q -- "--output-file"; then
        if [[ -n "$OUTPUT_FILE" ]]; then
            cmd+=("--output-file" "$OUTPUT_FILE")
            log_info "Output will be written to: $OUTPUT_FILE"
        elif [[ "$FORMAT" == "sarif" ]]; then
            # Default SARIF output file for GitHub Actions
            OUTPUT_FILE="mdbook-lint-results.sarif"
            cmd+=("--output-file" "$OUTPUT_FILE")
            log_info "SARIF output will be written to: $OUTPUT_FILE"
        fi
    else
        if [[ -n "$OUTPUT_FILE" ]]; then
            log_warn "--output-file not supported in this version, output will go to stdout"
        fi
    fi
    
    # Handle rules input
    if [[ -n "$RULES" ]]; then
        # Convert comma-separated rules to individual arguments
        IFS=',' read -ra RULES_ARRAY <<< "$RULES"
        for rule in "${RULES_ARRAY[@]}"; do
            rule=$(echo "$rule" | xargs) # trim whitespace
            if [[ "$rule" =~ ^-?[A-Z]+[0-9]+ ]]; then
                cmd+=("--rule" "$rule")
            fi
        done
        log_info "Custom rules: $RULES"
    fi
    
    # Add files/patterns
    IFS=' ' read -ra FILES_ARRAY <<< "$FILES"
    cmd+=("${FILES_ARRAY[@]}")
    
    log_debug "Command: ${cmd[*]}"
    echo "${cmd[@]}"
}

# Parse output and extract metrics
parse_output() {
    local output_content="$1"
    
    case "$FORMAT" in
        json)
            parse_json_output "$output_content"
            ;;
        sarif)
            parse_sarif_output "$output_content"
            ;;
        *)
            parse_human_output "$output_content"
            ;;
    esac
}

parse_json_output() {
    local content="$1"
    
    if command -v jq >/dev/null 2>&1; then
        VIOLATIONS_COUNT=$(echo "$content" | jq '.violations | length' 2>/dev/null || echo "0")
        WARNINGS_COUNT=$(echo "$content" | jq '[.violations[] | select(.level == "warning")] | length' 2>/dev/null || echo "0")
        ERRORS_COUNT=$(echo "$content" | jq '[.violations[] | select(.level == "error")] | length' 2>/dev/null || echo "0")
    else
        # Fallback parsing without jq
        VIOLATIONS_COUNT=$(echo "$content" | grep -o '"violations":\[' | wc -l || echo "0")
        WARNINGS_COUNT=$(echo "$content" | grep -o '"level":"warning"' | wc -l || echo "0")
        ERRORS_COUNT=$(echo "$content" | grep -o '"level":"error"' | wc -l || echo "0")
    fi
    
    # Sanitize variables to remove any whitespace/newlines
    VIOLATIONS_COUNT=$(echo "$VIOLATIONS_COUNT" | tr -d '\n\r\t ')
    WARNINGS_COUNT=$(echo "$WARNINGS_COUNT" | tr -d '\n\r\t ')
    ERRORS_COUNT=$(echo "$ERRORS_COUNT" | tr -d '\n\r\t ')
}

parse_sarif_output() {
    local content="$1"
    
    # SARIF file should be created at output location
    if [[ -n "$OUTPUT_FILE" && -f "$OUTPUT_FILE" ]]; then
        SARIF_FILE="$OUTPUT_FILE"
    else
        SARIF_FILE="mdbook-lint.sarif"
    fi
    
    # Try to parse SARIF for counts
    if command -v jq >/dev/null 2>&1 && [[ -f "$SARIF_FILE" ]]; then
        VIOLATIONS_COUNT=$(jq '[.runs[].results[]] | length' "$SARIF_FILE" 2>/dev/null || echo "0")
        WARNINGS_COUNT=$(jq '[.runs[].results[] | select(.level == "warning")] | length' "$SARIF_FILE" 2>/dev/null || echo "0")
        ERRORS_COUNT=$(jq '[.runs[].results[] | select(.level == "error")] | length' "$SARIF_FILE" 2>/dev/null || echo "0")
    fi
    
    # Sanitize variables to remove any whitespace/newlines
    VIOLATIONS_COUNT=$(echo "$VIOLATIONS_COUNT" | tr -d '\n\r\t ')
    WARNINGS_COUNT=$(echo "$WARNINGS_COUNT" | tr -d '\n\r\t ')
    ERRORS_COUNT=$(echo "$ERRORS_COUNT" | tr -d '\n\r\t ')
}

parse_human_output() {
    local content="$1"
    
    # Count violations from human-readable output and sanitize output
    VIOLATIONS_COUNT=$(echo "$content" | grep -c "^.*:.*:.*:" 2>/dev/null || echo "0")
    WARNINGS_COUNT=$(echo "$content" | grep -c "\[warning\]" 2>/dev/null || echo "0")
    ERRORS_COUNT=$(echo "$content" | grep -c "\[error\]" 2>/dev/null || echo "0")
    
    # Sanitize variables to remove any whitespace/newlines
    VIOLATIONS_COUNT=$(echo "$VIOLATIONS_COUNT" | tr -d '\n\r\t ')
    WARNINGS_COUNT=$(echo "$WARNINGS_COUNT" | tr -d '\n\r\t ')
    ERRORS_COUNT=$(echo "$ERRORS_COUNT" | tr -d '\n\r\t ')
    
    # If no explicit levels, assume all are warnings
    if [[ "$VIOLATIONS_COUNT" -gt 0 && "$WARNINGS_COUNT" -eq 0 && "$ERRORS_COUNT" -eq 0 ]]; then
        WARNINGS_COUNT="$VIOLATIONS_COUNT"
    fi
}

# Set GitHub Actions outputs
set_outputs() {
    if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
        echo "violations=$VIOLATIONS_COUNT" >> "$GITHUB_OUTPUT"
        echo "warnings=$WARNINGS_COUNT" >> "$GITHUB_OUTPUT"
        echo "errors=$ERRORS_COUNT" >> "$GITHUB_OUTPUT"
        
        if [[ -n "$SARIF_FILE" && -f "$SARIF_FILE" ]]; then
            echo "sarif-file=$SARIF_FILE" >> "$GITHUB_OUTPUT"
        fi
    fi
}

# Create annotations for GitHub Actions
create_annotations() {
    local output_content="$1"
    
    # Only create annotations for human format
    if [[ "$FORMAT" == "human" ]]; then
        echo "$output_content" | while IFS= read -r line; do
            if [[ "$line" =~ ^([^:]+):([0-9]+):([0-9]+):[[:space:]]*(.+)$ ]]; then
                local file="${BASH_REMATCH[1]}"
                local line_num="${BASH_REMATCH[2]}"
                local col="${BASH_REMATCH[3]}"
                local message="${BASH_REMATCH[4]}"
                
                # Determine annotation level
                local level="warning"
                if [[ "$message" =~ \[error\] ]]; then
                    level="error"
                fi
                
                echo "::$level file=$file,line=$line_num,col=$col::$message"
            fi
        done
    fi
}

# Main execution
main() {
    log_info "Running mdbook-lint..."
    log_debug "Files: $FILES"
    log_debug "Config: $CONFIG_FILE"
    log_debug "Format: $FORMAT"
    log_debug "Fail on warnings: $FAIL_ON_WARNINGS"
    
    setup_temp_output
    
    # Build and execute command
    local cmd_array
    read -ra cmd_array <<< "$(build_command)"
    
    local exit_code=0
    local output_content=""
    
    # Run mdbook-lint and capture output
    if [[ -n "$OUTPUT_FILE" ]]; then
        # Output to file
        "${cmd_array[@]}" 2>&1 | tee "$TEMP_OUTPUT" || exit_code=$?
        if [[ -f "$OUTPUT_FILE" ]]; then
            output_content=$(cat "$OUTPUT_FILE")
        fi
    else
        # Capture output
        "${cmd_array[@]}" 2>&1 | tee "$TEMP_OUTPUT" || exit_code=$?
        output_content=$(cat "$TEMP_OUTPUT")
    fi
    
    # Parse results
    parse_output "$output_content"
    
    # Set outputs
    set_outputs
    
    # Create annotations
    create_annotations "$output_content"
    
    # Report results
    log_info "Results:"
    log_info "  Total violations: $VIOLATIONS_COUNT"
    log_info "  Warnings: $WARNINGS_COUNT"
    log_info "  Errors: $ERRORS_COUNT"
    
    if [[ -n "$SARIF_FILE" && -f "$SARIF_FILE" ]]; then
        log_info "  SARIF file: $SARIF_FILE"
    fi
    
    # Determine final exit code
    local final_exit_code=0
    
    if [[ "$exit_code" -ne 0 ]]; then
        final_exit_code="$exit_code"
    elif [[ "$FAIL_ON_WARNINGS" == "true" && "$WARNINGS_COUNT" -gt 0 ]]; then
        log_warn "Failing due to warnings (fail-on-warnings=true)"
        final_exit_code=1
    elif [[ "$ERRORS_COUNT" -gt 0 ]]; then
        log_error "Failing due to errors"
        final_exit_code=1
    fi
    
    if [[ "$final_exit_code" -eq 0 ]]; then
        log_info "mdbook-lint completed successfully!"
    else
        log_error "mdbook-lint failed with exit code $final_exit_code"
    fi
    
    exit "$final_exit_code"
}

main "$@"