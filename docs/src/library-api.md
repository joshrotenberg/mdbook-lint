# Library API

mdbook-lint is designed as a library-first project, making it easy to integrate markdown linting capabilities into your own Rust applications.

## Overview

The core library (`mdbook-lint-core`) provides a clean, well-documented API for:

- Creating lint engines with different rule sets
- Processing markdown documents programmatically  
- Configuring rules and behavior
- Handling violations and results

## Quick Start

Add mdbook-lint to your `Cargo.toml`:

```toml
[dependencies]
mdbook-lint-core = "0.3.0"
```

Basic usage:

```rust
use mdbook_lint_core::{Document, create_engine_with_all_rules};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a lint engine with all available rules
    let engine = create_engine_with_all_rules();
    
    // Create a document from content and path
    let content = "# My Document\n\nSome content here.";
    let document = Document::new(content.to_string(), PathBuf::from("example.md"))?;
    
    // Lint the document
    let violations = engine.lint_document(&document)?;
    
    // Process results
    for violation in violations {
        println!("{}:{}: {} - {}", 
            violation.line, 
            violation.column,
            violation.rule_id, 
            violation.message
        );
    }
    
    Ok(())
}
```

## Core Types

### Document

Represents a markdown document with content and metadata:

```rust
use mdbook_lint_core::Document;
use std::path::PathBuf;

let document = Document::new(
    "# Title\n\nContent".to_string(),
    PathBuf::from("my-file.md")
)?;
```

### LintEngine

The main interface for linting operations:

```rust
use mdbook_lint_core::{LintEngine, create_engine_with_all_rules, create_standard_engine};

// Engine with all rules (standard + mdBook-specific)
let all_engine = create_engine_with_all_rules();

// Engine with only standard markdown rules (MD001-MD059)
let standard_engine = create_standard_engine();

// Engine with only mdBook-specific rules
let mdbook_engine = create_mdbook_engine();
```

### Configuration

Control which rules are enabled and configure their behavior:

```rust
use mdbook_lint_core::Config;

let mut config = Config::default();

// Enable specific rules only
config.enabled_rules = vec!["MD001".to_string(), "MD013".to_string()];

// Disable specific rules
config.disabled_rules = vec!["MD002".to_string()];

// Enable specific categories
config.enabled_categories = vec!["structure".to_string()];

// Lint with configuration
let violations = engine.lint_document_with_config(&document, &config)?;
```

### Violations

Results from linting operations:

```rust
use mdbook_lint_core::{Violation, Severity};

// Violations contain detailed information about issues found
for violation in violations {
    println!("Rule: {}", violation.rule_id);
    println!("Message: {}", violation.message);
    println!("Location: {}:{}", violation.line, violation.column);
    
    match violation.severity {
        Severity::Error => println!("This is an error"),
        Severity::Warning => println!("This is a warning"),
        Severity::Info => println!("This is informational"),
    }
}
```

## Advanced Usage

### Custom Rule Providers

Create engines with specific rule sets:

```rust
use mdbook_lint_core::{PluginRegistry, StandardRuleProvider};

let mut registry = PluginRegistry::new();
registry.register_provider(Box::new(StandardRuleProvider))?;

let engine = registry.create_engine()?;
```

### Error Handling

The library uses `anyhow` for comprehensive error handling:

```rust
use mdbook_lint_core::error::Result;

fn lint_file(path: &Path) -> Result<Vec<Violation>> {
    let content = std::fs::read_to_string(path)?;
    let document = Document::new(content, path.to_path_buf())?;
    
    let engine = create_engine_with_all_rules();
    engine.lint_document(&document)
}
```

### Batch Processing

Process multiple documents efficiently:

```rust
use walkdir::WalkDir;

let engine = create_engine_with_all_rules();
let mut all_violations = Vec::new();

for entry in WalkDir::new("src/") {
    let entry = entry?;
    if entry.path().extension().map_or(false, |ext| ext == "md") {
        let content = std::fs::read_to_string(entry.path())?;
        let document = Document::new(content, entry.path().to_path_buf())?;
        
        let violations = engine.lint_document(&document)?;
        all_violations.extend(violations);
    }
}
```

## Rule Categories

Rules are organized into logical categories:

- **Structure**: Document structure and hierarchy (MD001, MD003, etc.)
- **Formatting**: Code blocks, lists, emphasis (MD004, MD005, etc.)  
- **Content**: Language, spelling, accessibility (MD044, MD045, etc.)
- **Links**: URL validation and formatting (MD034, MD039, etc.)
- **MdBook**: mdBook-specific checks (MDBOOK001-004)

Enable categories programmatically:

```rust
let mut config = Config::default();
config.enabled_categories = vec![
    "structure".to_string(),
    "formatting".to_string()
];
```

## Integration Examples

### mdBook Preprocessor

```rust
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::book::Book;
use mdbook_lint_core::create_engine_with_all_rules;

struct MyLinter {
    engine: LintEngine,
}

impl Preprocessor for MyLinter {
    fn run(&self, ctx: &PreprocessorContext, book: Book) -> mdbook::errors::Result<Book> {
        // Lint each chapter
        // Return book unchanged (linting only)
        Ok(book)
    }
}
```

### CI/CD Integration  

```rust
use mdbook_lint_core::{create_engine_with_all_rules, Severity};

fn main() -> std::process::ExitCode {
    let engine = create_engine_with_all_rules();
    let mut has_errors = false;
    
    // Process all markdown files in repository
    // Set exit code based on results
    
    if has_errors {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
```

## API Documentation

For complete API documentation with examples, see the [rustdoc documentation](https://docs.rs/mdbook-lint-core):

- **Online**: https://docs.rs/mdbook-lint-core/latest/mdbook_lint_core/
- **Local**: Run `cargo doc --open --no-deps` in the repository

The rustdoc includes:

- Complete API reference with examples
- Module-level documentation
- Implementation details and internal architecture
- Links between related types and functions

## Performance Considerations

- **Single-pass parsing**: Documents are parsed once and reused across all rules
- **Lazy evaluation**: Rules are only applied to relevant document sections  
- **Memory efficient**: Minimal AST retention, streaming for large files
- **Parallel processing**: Use `rayon` or similar for batch operations

## Error Types

The library defines specific error types for different failure modes:

- `ConfigError`: Configuration parsing and validation issues
- `DocumentError`: Document creation and parsing problems  
- `RuleError`: Rule execution failures
- `IoError`: File system access problems

## Next Steps

- See [Configuration](./configuration.md) for detailed configuration options
- Check [Rules Reference](./rules-reference.md) for available rules
- Review [Architecture](./architecture.md) for internal design details
- Browse the source code on [GitHub](https://github.com/joshrotenberg/mdbook-lint)