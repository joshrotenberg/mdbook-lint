# Contributing

Thank you for your interest in contributing to mdbook-lint! This guide will help you get started with contributing to the project.

## Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a branch** for your changes
4. **Make your changes** and add tests
5. **Run tests** to ensure everything works
6. **Submit a pull request**

## Development Setup

### Prerequisites

- Rust 1.88 or later
- Git

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/mdbook-lint.git
cd mdbook-lint

# Build the project
cargo build

# Run tests
cargo test

# Check code style
cargo clippy
cargo fmt --check
```

## Project Structure

```
mdbook-lint/
├── src/                 # Source code
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library entry point
│   ├── rules/          # Linting rules
│   └── config/         # Configuration handling
├── tests/              # Integration tests
├── docs/               # Documentation (this site)
└── scripts/            # Development scripts
```

## Making Changes

### Code Style

- Follow Rust standard formatting (use `cargo fmt`)
- Ensure code passes clippy checks (`cargo clippy`)
- Write clear, self-documenting code
- Add comments for complex logic

### Testing

All contributions should include appropriate tests:

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run corpus tests (if applicable)
cargo test corpus
```

### Adding New Rules

To add a new linting rule:

1. Create a new file in `src/rules/`
2. Implement the rule logic
3. Add tests in `tests/rules/`
4. Update rule registry
5. Add documentation

Example rule structure:
```rust
use crate::rule::{Rule, RuleResult};

pub struct NewRule;

impl Rule for NewRule {
    fn check(&self, document: &Document) -> RuleResult {
        // Rule implementation
    }
}
```

## Documentation

- Update documentation for new features
- Ensure examples are working and tested
- Use clear, concise language
- Follow the existing documentation style

### Building Documentation

```bash
cd docs
mdbook build
mdbook serve  # For local preview
```

## Submitting Changes

### Pull Request Guidelines

1. **Create a focused PR** - one feature or fix per PR
2. **Write a clear title** - describe what the PR does
3. **Provide description** - explain the changes and motivation
4. **Include tests** - ensure new code is tested
5. **Update documentation** - if applicable

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement

## Testing
- [ ] Tests pass locally
- [ ] Added new tests for changes
- [ ] Updated documentation

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated if needed
```

## Code Review Process

1. **Automated checks** run on all PRs
2. **Maintainer review** for code quality and design
3. **Testing** to ensure changes work as expected
4. **Merge** once approved and tests pass

## Reporting Issues

When reporting bugs or requesting features:

1. **Search existing issues** first
2. **Use issue templates** when available
3. **Provide clear reproduction steps** for bugs
4. **Include environment information**

### Bug Report Template

```markdown
**Description**
Clear description of the bug

**To Reproduce**
Steps to reproduce the behavior

**Expected Behavior**
What you expected to happen

**Environment**
- mdbook-lint version:
- Rust version:
- OS:
```

## Community Guidelines

- Be respectful and inclusive
- Help others learn and contribute
- Provide constructive feedback
- Follow the project's code of conduct

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check the docs first

## Recognition

Contributors are recognized in:
- GitHub contributors list
- Release notes for significant contributions
- Project documentation

Thank you for contributing to mdbook-lint!