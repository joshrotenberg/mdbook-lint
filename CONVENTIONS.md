# Project Conventions

This document outlines the essential naming conventions and standards for mdbook-lint.

## Conventional Commits

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[scope]: <description>

feat(rules): add MD040 rule for fenced code blocks
fix(cli): handle empty files correctly
docs: update installation instructions
refactor(engine): simplify rule registry
```

### Types
- **feat**: New features
- **fix**: Bug fixes
- **docs**: Documentation changes
- **test**: Adding or updating tests
- **refactor**: Code changes that neither fix bugs nor add features
- **perf**: Performance improvements
- **chore**: Maintenance tasks
- **ci**: CI/CD changes

### Scopes
- **rules**: Linting rules
- **cli**: Command line interface
- **config**: Configuration system
- **engine**: Core linting engine
- **docs**: Documentation

## Branch Naming

```
<type>/<description>

feature/md040-code-block-language
fix/empty-file-handling
docs/contributing-guide
refactor/rule-registry
```

## Code Naming

### Rust Conventions
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

## Configuration

### Keys
Use `kebab-case` for all configuration keys:

```toml
fail-on-warnings = true
enabled-rules = ["MD001", "MD013"]
disabled-categories = ["style"]

[rules.MD013]
line-length = 100
ignore-code-blocks = true
```

### Files
- **Config files**: `.mdbook-lint.toml`, `.mdbook-lint.yaml`, `.mdbook-lint.json`
- **Documentation**: `kebab-case.md`

## Quality Standards

### Required Checks
```bash
cargo test     # All tests must pass
cargo fmt      # Code must be formatted
cargo clippy   # No warnings allowed
```

### Documentation Style
- Simple, clear, and factual
- No marketing language or emojis
- Include working code examples
- Professional tone throughout

This document covers the essential conventions. For complete contributing information, see the [Contributing Guide](https://joshrotenberg.github.io/mdbook-lint/contributing.html).