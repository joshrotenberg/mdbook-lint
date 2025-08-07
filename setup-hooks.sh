#!/bin/bash
#
# Setup git hooks for mdbook-lint
# This configures git to use our custom hooks
#

set -e

echo "🔧 Setting up git hooks for mdbook-lint..."

# Configure git to use our custom hooks directory
git config core.hooksPath .githooks

echo "✅ Git hooks configured successfully!"
echo ""
echo "The following checks will now run automatically before each commit:"
echo "  • Code formatting (cargo fmt)"
echo "  • Linting (cargo clippy)"
echo "  • Unit tests (cargo test --lib)"
echo "  • Integration tests (cargo test --test '*')"  
echo "  • Doc tests (cargo test --doc)"
echo "  • Check for Claude Code signatures"
echo "  • Check for emojis"
echo ""
echo "To bypass these checks (not recommended), use: git commit --no-verify"