# Project Conventions

This document outlines the naming conventions and standards for the mdbook-lint project.

## Conventional Commits

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Scopes

- **cli**: Command line interface
- **preprocessor**: mdBook preprocessor functionality
- **rules**: Linting rules
- **config**: Configuration system
- **engine**: Core linting engine
- **tests**: Test infrastructure
- **docs**: Documentation

### Examples

```bash
feat(rules): add MD040 rule for fenced code blocks
fix(cli): handle empty files correctly
docs: update installation instructions
test(corpus): add edge case for nested lists
chore(deps): update comrak to 0.21
ci: add codecov integration
```

## Branch Naming

### Format

```
<type>/<short-description>
```

### Types

- **feature**: New features
- **bugfix**: Bug fixes
- **hotfix**: Critical fixes for production
- **docs**: Documentation changes
- **chore**: Maintenance tasks
- **refactor**: Code refactoring

### Examples

```bash
feature/md040-code-block-language
bugfix/empty-file-handling
hotfix/security-vulnerability
docs/installation-guide
chore/update-dependencies
refactor/rule-registry-cleanup
```

## Pull Request Naming

### Format

```
<type>[scope]: <description>
```

### Examples

```bash
feat(rules): Add MD040 rule for fenced code blocks
fix(cli): Handle empty files correctly
docs: Update installation instructions
```

## Issue Naming

### Format

```
<type>: <clear description>
```

### Examples

```bash
bug: CLI crashes on empty markdown files
feature: Add support for custom rule configurations
docs: Missing examples in README
enhancement: Improve error messages for rule violations
```

## Release Naming

### Version Format

We follow [Semantic Versioning](https://semver.org/) (SemVer):

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

### Release Tags

```bash
v1.0.0
v1.1.0
v1.1.1
```

### Pre-release Versions

```bash
v1.0.0-alpha.1
v1.0.0-beta.1
v1.0.0-rc.1
```

## File and Directory Naming

### Source Files

- Use `snake_case` for Rust files: `rule_registry.rs`
- Use `kebab-case` for configuration files: `mdbook-lint.toml`
- Use `PascalCase` for struct/enum names: `RuleRegistry`
- Use `SCREAMING_SNAKE_CASE` for constants: `DEFAULT_CONFIG_FILE`

### Test Files

- Unit tests: Same file with `#[cfg(test)]` module
- Integration tests: `tests/integration_*.rs`
- Test fixtures: `tests/fixtures/valid_markdown.md`

### Documentation Files

- Use `UPPERCASE` for root docs: `README.md`, `CONTRIBUTING.md`
- Use `kebab-case` for guides: `installation-guide.md`
- Use clear, descriptive names: `rule-configuration.md`

## Rule Naming

### Standard Rules

Follow markdownlint convention:
- **ID**: `MD###` (e.g., `MD001`, `MD040`)
- **Name**: `kebab-case` (e.g., `heading-increment`, `code-block-language`)
- **File**: `md###.rs` (e.g., `md001.rs`, `md040.rs`)

### mdBook Rules

Follow our convention:
- **ID**: `MDBOOK###` (e.g., `MDBOOK001`, `MDBOOK002`)
- **Name**: `kebab-case` (e.g., `code-block-language`, `internal-link-validation`)
- **File**: `mdbook###.rs` (e.g., `mdbook001.rs`, `mdbook002.rs`)

## Configuration Naming

### Configuration Keys

Use `kebab-case` for configuration keys:

```toml
fail-on-warnings = true
enabled-categories = ["structure", "formatting"]
disabled-rules = ["MD013", "MD033"]

[rules.MD013]
line-length = 100
ignore-code-blocks = true
```

### Environment Variables

Use `SCREAMING_SNAKE_CASE` with `MDBOOK_LINT_` prefix:

```bash
MDBOOK_LINT_CONFIG_FILE=/path/to/config.toml
MDBOOK_LINT_FAIL_ON_WARNINGS=true
```

## Error and Log Messages

### Format

Keep messages clear and actionable:

```rust
// Good
"Missing language tag for code block at line 15"
"Configuration file not found: .mdbook-lint.toml"
"Rule MD013 failed: line too long (150 > 100 characters)"

// Avoid
"Error occurred"
"Invalid input"
"Something went wrong"
```

### Log Levels

- **error**: Critical failures that prevent operation
- **warn**: Issues that don't prevent operation but should be addressed
- **info**: General information about operation
- **debug**: Detailed information for debugging

## Documentation Style

### Tone

- **Simple and factual**: No marketing language
- **Clear and concise**: Get to the point quickly  
- **Professional**: Assume technical audience
- **Helpful**: Include examples where useful

### Structure

```markdown
# Title

Brief description of what this does.

## Usage

Basic usage example.

## Configuration

Configuration options if applicable.

## Examples

Practical examples.
```

### Code Examples

Always include working examples:

```rust
use mdbook_lint::{create_engine_with_all_rules, Document};

let engine = create_engine_with_all_rules();
let document = Document::new("# Hello".to_string(), "test.md".into())?;
let violations = engine.lint_document(&document)?;
```

## Testing Conventions

### Test Naming

```rust
#[test]
fn test_md001_valid_sequence() { }

#[test]  
fn test_md001_skip_level() { }

#[test]
fn test_config_from_toml() { }
```

### Test Organization

- **Unit tests**: In same file as implementation
- **Integration tests**: In `tests/` directory
- **Fixtures**: In `tests/fixtures/` directory
- **Corpus tests**: For compatibility validation

### Test Content

- Test both positive and negative cases
- Use descriptive assertions
- Include edge cases
- Keep tests focused and independent

## Labels and Project Management

### Issue Labels

Use our label system for categorization:
- **Type**: `type: feat`, `type: fix`, `type: docs`
- **Component**: `component: cli`, `component: rules`
- **Priority**: `priority: high`, `priority: medium`
- **Status**: `status: needs-review`, `status: blocked`

### Milestones

Use semantic version numbers:
- `v0.2.0` - Next minor release
- `v0.1.1` - Next patch release
- `v1.0.0` - Major release milestone

## Review Guidelines

### Pull Request Requirements

- [ ] Follows conventional commit format
- [ ] Includes tests for new functionality
- [ ] Updates documentation if needed
- [ ] Passes all CI checks
- [ ] Has clear description and examples

### Code Review Focus

- **Correctness**: Does it work as intended?
- **Clarity**: Is the code easy to understand?
- **Consistency**: Follows project conventions?
- **Testing**: Adequate test coverage?
- **Documentation**: Clear and up-to-date?

This document is a living guide - update it as the project evolves while maintaining consistency and clarity.