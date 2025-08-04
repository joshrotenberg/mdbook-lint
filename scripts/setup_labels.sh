#!/bin/bash
set -euo pipefail

# GitHub Labels Setup Script for mdbook-lint
# This script creates a comprehensive set of labels for project management

REPO="joshrotenberg/mdbook-lint"

echo "Setting up GitHub labels for $REPO..."

# Function to create or update a label
create_label() {
    local name="$1"
    local color="$2"
    local description="$3"

    echo "Creating label: $name"
    gh label create "$name" --color "$color" --description "$description" --force 2>/dev/null || true
}

# Type Labels (Conventional Commits)
create_label "type: feat" "0e8a16" "New feature or enhancement"
create_label "type: fix" "d73a4a" "Bug fix"
create_label "type: docs" "0366d6" "Documentation changes"
create_label "type: style" "f9d0c4" "Code style changes (formatting, etc.)"
create_label "type: refactor" "fbca04" "Code refactoring"
create_label "type: perf" "d4c5f9" "Performance improvements"
create_label "type: test" "c5def5" "Adding or fixing tests"
create_label "type: chore" "fef2c0" "Maintenance tasks"
create_label "type: ci" "bfd4f2" "CI/CD changes"
create_label "type: build" "1d76db" "Build system changes"
create_label "type: revert" "e99695" "Reverting previous changes"

# Priority Labels
create_label "priority: critical" "b60205" "Critical issue requiring immediate attention"
create_label "priority: high" "d93f0b" "High priority issue"
create_label "priority: medium" "fbca04" "Medium priority issue"
create_label "priority: low" "0e8a16" "Low priority issue"

# Status Labels
create_label "status: needs-review" "006b75" "Waiting for review"
create_label "status: needs-changes" "c2e0c6" "Changes requested"
create_label "status: blocked" "d73a4a" "Blocked by another issue or dependency"
create_label "status: in-progress" "0e8a16" "Currently being worked on"
create_label "status: ready-to-merge" "0e8a16" "Ready for merge"

# Component Labels
create_label "component: cli" "5319e7" "Command line interface"
create_label "component: preprocessor" "1d76db" "mdBook preprocessor functionality"
create_label "component: rules" "0052cc" "Linting rules (MD001-059, MDBOOK001-004)"
create_label "component: config" "c5def5" "Configuration system"
create_label "component: engine" "0366d6" "Core linting engine"
create_label "component: tests" "f9d0c4" "Test infrastructure"
create_label "component: docs" "0366d6" "Documentation"
create_label "component: ci" "bfd4f2" "Continuous integration"

# Rule-Specific Labels
create_label "rules: standard" "0052cc" "Standard markdown rules (MD001-MD059)"
create_label "rules: mdbook" "1d76db" "mdBook-specific rules (MDBOOK001-004)"
create_label "rules: new" "0e8a16" "Proposing new rules"
create_label "rules: compatibility" "fbca04" "markdownlint compatibility issues"

# Effort Labels
create_label "effort: small" "c2e0c6" "Small effort (< 2 hours)"
create_label "effort: medium" "fbca04" "Medium effort (2-8 hours)"
create_label "effort: large" "d93f0b" "Large effort (> 8 hours)"
create_label "effort: epic" "b60205" "Epic requiring multiple PRs"

# Good First Issue Labels
create_label "good first issue" "7057ff" "Good for newcomers"
create_label "help wanted" "008672" "Extra attention is needed"

# Special Labels
create_label "breaking change" "b60205" "Introduces breaking changes"
create_label "dependencies" "0366d6" "Pull requests that update dependencies"
create_label "security" "d73a4a" "Security-related issues"
create_label "performance" "d4c5f9" "Performance-related issues"
create_label "enhancement" "a2eeef" "Feature request or improvement"
create_label "question" "cc317c" "Further information is requested"
create_label "wontfix" "ffffff" "This will not be worked on"
create_label "duplicate" "cfd3d7" "This issue or pull request already exists"
create_label "invalid" "e4e669" "This doesn't seem right"

# Release Labels
create_label "release: major" "b60205" "Major version release"
create_label "release: minor" "0e8a16" "Minor version release"
create_label "release: patch" "c2e0c6" "Patch version release"

# Platform Labels
create_label "platform: windows" "0052cc" "Windows-specific issues"
create_label "platform: macos" "1d76db" "macOS-specific issues"
create_label "platform: linux" "0e8a16" "Linux-specific issues"

echo ""
echo "✅ GitHub labels setup complete!"
echo ""
echo "Labels created for:"
echo "  • Conventional commit types"
echo "  • Priority levels"
echo "  • Component areas"
echo "  • Status tracking"
echo "  • Effort estimation"
echo "  • Special categories"
echo ""
echo "To view all labels: gh label list"
