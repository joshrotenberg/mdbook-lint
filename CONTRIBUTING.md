# Contributing to mdbook-lint

Thank you for your interest in contributing to mdbook-lint! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.88.0 or later
- Git

### Setting Up the Development Environment

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/mdbook-lint.git
   cd mdbook-lint
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run the tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Making Changes

1. Create a new branch for your feature or bug fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes
3. Add tests for new functionality
4. Ensure all tests pass:
   ```bash
   cargo test
   ```
5. Check code formatting:
   ```bash
   cargo fmt --all -- --check
   ```
6. Run clippy for additional linting:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/) format:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for adding tests
- `refactor:` for code refactoring
- `perf:` for performance improvements
- `chore:` for maintenance tasks

Examples:
```
feat: add MD040 rule for fenced code blocks language
fix: handle empty files in document parser
docs: update README with installation instructions
```

## Code Guidelines

### Architecture

- Rules should implement the `Rule` or `AstRule` trait
- Use `AstRule` for rules that need AST access, `Rule` for line-based rules
- Add comprehensive tests for all rule logic
- Follow the established error handling patterns

### Adding New Rules

1. Create a new file in `src/rules/standard/` (e.g., `md999.rs`)
2. Implement the appropriate trait:
   ```rust
   use crate::rule::{AstRule, RuleCategory, RuleMetadata};
   use crate::{Document, violation::{Severity, Violation}};
   use comrak::nodes::AstNode;

   pub struct MD999;

   impl AstRule for MD999 {
       fn id(&self) -> &'static str {
           "MD999"
       }

       fn name(&self) -> &'static str {
           "rule-name"
       }

       fn description(&self) -> &'static str {
           "Description of what this rule checks"
       }

       fn metadata(&self) -> RuleMetadata {
           RuleMetadata::stable(RuleCategory::Structure)
       }

       fn check_with_ast<'a>(
           &self,
           document: &Document,
           ast: &'a AstNode<'a>,
       ) -> crate::error::Result<Vec<Violation>> {
           // Implementation here
           Ok(vec![])
       }
   }
   ```
3. Add the module to `src/rules/standard/mod.rs`
4. Register the rule in `StandardRuleProvider`
5. Add comprehensive tests
6. Update documentation

### Testing

- Add unit tests in the same file as your rule
- Use the `#[cfg(test)]` module pattern
- Create test fixtures for complex scenarios
- Run corpus tests to ensure compatibility: `cargo test corpus`
- Ensure all tests pass: `cargo test`

### Documentation

Keep documentation simple and factual:
- Focus on what the rule does, not performance claims
- Include clear examples of violations
- Document configuration options if any
- Use the existing documentation style

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test '*'
```

### Corpus Tests (with markdownlint comparison)
```bash
cargo test corpus_integration_test -- --ignored
```

## Pull Request Process

1. Update documentation as needed
2. Add or update tests for your changes
3. Ensure all CI checks pass
4. Create a pull request with:
   - Clear title and description
   - Reference any related issues
   - Include examples of the change in action

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Corpus tests pass (if applicable)

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
```

## Code Style

### Rust Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `cargo clippy` and address all warnings
- Prefer explicit error handling over `.unwrap()`
- Document public APIs with rustdoc comments
- Keep functions focused and reasonably sized

### Documentation Style

- Keep rustdoc comments simple and factual
- Avoid marketing language or performance claims
- Include examples when helpful
- Explain complex logic with inline comments

## Issue Reporting

### Bug Reports

Include:
- Rust version
- Operating system
- Minimal reproduction example
- Expected vs actual behavior
- Relevant error messages

### Feature Requests

Include:
- Use case description
- Proposed API or interface
- Examples of how it would be used
- Any implementation ideas

## Community

- Be respectful and constructive
- Help others learn and contribute
- Ask questions in issues or discussions

Thank you for contributing to mdbook-lint!