# Contributing to mdbook-lint

Thank you for your interest in contributing to mdbook-lint! This guide covers everything you need to know to contribute effectively.

## Quick Start

### Prerequisites

- Rust 1.88.0 or later
- Git

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/mdbook-lint.git
cd mdbook-lint

# Set up development environment
cargo build
cargo test
cargo fmt
cargo clippy
```

### Making Your First Contribution

1. **Create a branch**: `git checkout -b feature/your-change`
2. **Make changes**: Follow the guidelines below
3. **Test thoroughly**: Ensure all tests pass
4. **Submit PR**: Use conventional commit format

## Project Structure

```
mdbook-lint/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library entry point
│   ├── engine.rs            # Core linting engine
│   ├── config.rs            # Configuration system
│   ├── document.rs          # Markdown document processing
│   ├── rule.rs              # Rule trait definitions
│   ├── rules/               # Rule implementations
│   │   ├── standard/        # Standard markdown rules (MD001-MD059)
│   │   ├── mdbook001.rs     # mdBook-specific rules
│   │   └── ...
│   └── preprocessor.rs      # mdBook preprocessor integration
├── tests/                   # Integration tests
├── docs/                    # This documentation site
└── scripts/                 # Development utilities
```

## Code Standards

### Rust Style

- **Formatting**: Use `cargo fmt` (enforced in CI)
- **Linting**: Fix all `cargo clippy` warnings
- **Error Handling**: Use `Result<T>` types, avoid `.unwrap()`
- **Documentation**: Document all public APIs with rustdoc
- **Testing**: Comprehensive unit and integration tests required

### Commit Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[scope]: <description>

feat(rules): add MD040 rule for fenced code blocks
fix(cli): handle empty files correctly
docs: update installation instructions
test: add edge cases for rule validation
refactor: simplify config parsing logic
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`
**Scopes**: `rules`, `cli`, `config`, `engine`, `docs`, `tests`

### Branch Naming

```
<type>/<description>

feature/md040-code-block-language
fix/empty-file-handling
docs/contributing-guide
refactor/rule-registry-cleanup
```

## Adding New Rules

### Rule Types

**Line-based Rules** (implement `Rule` trait):
- Simple checks on raw text lines
- Faster execution, lower memory usage
- Good for formatting rules

**AST-based Rules** (implement `AstRule` trait):
- Complex semantic analysis
- Full markdown structure access
- Required for structural rules

### Implementation Example

```rust
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{Document, violation::{Severity, Violation}};
use comrak::nodes::{AstNode, NodeValue};

pub struct MD999;

impl AstRule for MD999 {
    fn id(&self) -> &'static str {
        "MD999"
    }

    fn name(&self) -> &'static str {
        "example-rule"
    }

    fn description(&self) -> &'static str {
        "Example rule for demonstration"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure)
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
    ) -> crate::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        
        // Walk AST and check for violations
        for node in ast.descendants() {
            if let NodeValue::Heading(heading) = &node.data.borrow().value {
                if heading.level > 6 {
                    if let Some((line, col)) = document.node_position(node) {
                        violations.push(self.create_violation(
                            "Heading level exceeds maximum".to_string(),
                            line,
                            col,
                            Severity::Error,
                        ));
                    }
                }
            }
        }
        
        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md999_valid_headings() {
        let content = "# H1\n## H2\n### H3\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD999;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md999_detects_violations() {
        let content = "####### Invalid heading level";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD999;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD999");
    }
}
```

### Rule Registration

1. **Add module**: Include your rule in `src/rules/standard/mod.rs`
2. **Register rule**: Add to `StandardRuleProvider::register_rules()`
3. **Add to rule list**: Include ID in `StandardRuleProvider::rule_ids()`

### Testing Requirements

**Comprehensive testing is required**:
- Test valid cases (no violations)
- Test violation detection
- Test edge cases (empty files, very long lines, unicode)
- Test configuration options (if applicable)
- Test error conditions

## Configuration System

### Rule Configuration

Rules can accept configuration through `rule_config`:

```rust
fn check_ast(&self, document: &Document, ast: &AstNode) -> Result<Vec<Violation>> {
    let config = document.config.rule_config
        .get(self.id())
        .and_then(|v| v.as_object());
        
    let max_length = config
        .and_then(|c| c.get("max-length"))
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;
        
    // Use configuration in rule logic
}
```

### Supported Formats

Configuration files can be TOML, YAML, or JSON:

```toml
# .mdbook-lint.toml
fail-on-warnings = true
enabled-rules = ["MD001", "MD013"]

[rules.MD013]
line-length = 120
ignore-code-blocks = true
```

## CLI Development

### Adding Commands

Add new commands to the `Commands` enum in `src/main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    /// Lint markdown files
    Lint {
        files: Vec<String>,
        #[arg(short, long)]
        config: Option<String>,
    },
    
    /// Your new command
    NewCommand {
        input: PathBuf,
        #[arg(long)]
        option: bool,
    },
}
```

### Command Implementation

Create handler functions with proper error handling:

```rust
fn run_new_command(input: PathBuf, option: bool) -> Result<()> {
    // Validate input
    if !input.exists() {
        return Err(MdBookLintError::config_error(
            format!("Path does not exist: {}", input.display())
        ));
    }
    
    // Implementation
    Ok(())
}
```

## Testing

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --test '*'

# All tests
cargo test

# Specific test
cargo test test_name -- --exact

# With output
cargo test test_name -- --nocapture
```

### Test Organization

- **Unit tests**: In same file with `#[cfg(test)]`
- **Integration tests**: In `tests/` directory
- **CLI tests**: Use `assert_cmd` crate
- **Fixtures**: Test data in `tests/fixtures/`

### Writing Good Tests

```rust
#[test]
fn test_descriptive_name() {
    // Arrange
    let input = "test input";
    let expected = "expected output";
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Pull Request Guidelines

### Before Submitting

- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Documentation updated (if needed)
- [ ] Commit follows conventional format

### PR Template

```markdown
## Description
Brief description of changes and motivation

## Type of Change
- [ ] Bug fix
- [ ] New feature  
- [ ] Documentation update
- [ ] Refactoring

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style
- [ ] Self-review completed
- [ ] Documentation updated
```

### Review Process

1. **Automated checks** run on all PRs
2. **Maintainer review** for code quality
3. **Testing** to ensure functionality
4. **Merge** when approved and passing

## Architecture Overview

### Core Components

**LintEngine** (`src/engine.rs`):
- Orchestrates linting process
- Manages rule execution
- Aggregates results

**Rule System** (`src/rule.rs`, `src/rules/`):
- Defines `Rule` and `AstRule` traits
- Implements linting logic
- Categorizes by type and stability

**Document Processing** (`src/document.rs`):
- Parses markdown using comrak
- Provides position tracking
- Handles various formats

**Configuration** (`src/config.rs`):
- Multi-format support (TOML/YAML/JSON)
- Rule-specific settings
- Precedence handling

### Data Flow

```
Input Files → Document Parser → Lint Engine → Rules → Violations → Output
     ↓              ↓              ↓          ↓         ↓         ↓
  .md files    AST + Lines    Rule Registry  Checks   Results   CLI/JSON
```

## Common Tasks

### Debug Rule Issues

```bash
# Enable debug logging
export RUST_LOG=mdbook_lint=debug

# Run with backtrace
export RUST_BACKTRACE=1

# Test specific rule
cargo test md001 -- --nocapture
```

### Profile Performance

```bash
# Install tools
cargo install flamegraph

# Profile application
cargo build --release
sudo flamegraph -- ./target/release/mdbook-lint lint large-file.md
```

### Update Dependencies

```bash
# Check for updates
cargo outdated

# Update Cargo.lock
cargo update

# Update Cargo.toml versions
cargo upgrade
```

## Project Conventions

### Naming Standards

- **Files**: `snake_case.rs`
- **Structs/Enums**: `PascalCase`
- **Functions/Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Rules**: `MD###` or `MDBOOK###`

### Documentation Style

- Simple, clear, and factual
- Include working code examples
- No marketing language
- Professional tone throughout
- Link to related documentation

### Error Messages

- Clear and actionable
- Include relevant context
- Suggest fixes when possible
- Consistent formatting

Example:
```
"Missing language tag for code block at line 15, column 1
Consider adding a language identifier: ```rust"
```

## Release Process

### Versioning

We use [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

### Release Workflow

1. Changes merged to main via PR
2. Release-please creates release PR automatically
3. Merge release PR to trigger release
4. GitHub Actions publishes to crates.io

## Getting Help

### Resources

- **Documentation**: [joshrotenberg.github.io/mdbook-lint](https://joshrotenberg.github.io/mdbook-lint)
- **Repository**: [github.com/joshrotenberg/mdbook-lint](https://github.com/joshrotenberg/mdbook-lint)
- **Issues**: Report bugs and request features
- **Discussions**: Ask questions and get help

### Common Questions

**Q: How do I add a new rule?**
A: Follow the "Adding New Rules" section above. Start with the rule template and add comprehensive tests.

**Q: Why did my PR fail CI?**
A: Check that `cargo test`, `cargo fmt`, and `cargo clippy` all pass locally.

**Q: How do I test mdBook integration?**
A: Use the `MdBookLint` preprocessor in your tests. See existing integration tests for examples.

**Q: Can I contribute documentation improvements?**
A: Absolutely! Documentation improvements are highly valued. Edit the files in `docs/src/`.

## Community Guidelines

- Be respectful and constructive
- Help others learn and contribute
- Follow our professional standards
- Ask questions when unclear
- Provide helpful feedback in reviews

## Project Conventions

### Commit Format
We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[scope]: <description>

feat(rules): add MD040 rule for fenced code blocks
fix(cli): handle empty files correctly
docs: update installation instructions
refactor(engine): simplify rule registry
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`
**Scopes**: `rules`, `cli`, `config`, `engine`, `docs`

### Branch Naming
```
<type>/<description>

feature/md040-code-block-language
fix/empty-file-handling
docs/contributing-guide
refactor/rule-registry
```

### Code Naming
- **Files**: `snake_case.rs`
- **Structs/Enums**: `PascalCase`
- **Functions**: `snake_case`
- **Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`

### Rule Naming
- **Standard rules**: `MD###` (MD001, MD040, etc.)
- **mdBook rules**: `MDBOOK###` (MDBOOK001, MDBOOK002, etc.)
- **Rule files**: `md###.rs` or `mdbook###.rs`
- **Rule names**: `kebab-case` (heading-increment, code-block-language)

### Configuration Keys
Use `kebab-case` for all configuration keys:

```toml
fail-on-warnings = true
enabled-rules = ["MD001", "MD013"]
disabled-categories = ["style"]

[rules.MD013]
line-length = 100
ignore-code-blocks = true
```

### Documentation Style
- Simple, clear, and factual
- No marketing language or emojis
- Include working code examples
- Professional tone throughout

Thank you for contributing to mdbook-lint! Your efforts help make documentation better for everyone.