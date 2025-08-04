# Architecture

This page provides an overview of mdbook-lint's internal architecture and design decisions.

## High-Level Architecture

mdbook-lint is built as a modular Rust application with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Interface │    │  mdBook Plugin  │    │  Library Core   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Lint Engine   │
                    └─────────────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
  │ Rule Registry │  │ Configuration │  │ File Parser   │
  └───────────────┘  └───────────────┘  └───────────────┘
          │
     ┌────┴────┐
     │  Rules  │
     └─────────┘
```

## Core Components

### Lint Engine
The central component that orchestrates the linting process:
- Loads and validates configuration
- Discovers and parses markdown files
- Applies rules to parsed content
- Collects and formats results

### Rule System
A plugin-like architecture for linting rules:
- Each rule implements a common `Rule` trait
- Rules are automatically registered at compile time
- Configurable rule parameters
- Extensible for new rule types

### Parser
Handles markdown parsing and AST generation:
- Uses `comrak` for CommonMark compliance
- Maintains source position information
- Provides structured access to document elements

### Configuration
Flexible configuration system:
- TOML-based configuration files
- Command-line argument override
- Environment variable support
- Validation and error reporting

## Data Flow

1. **Input Processing**
   - Command-line arguments parsed
   - Configuration files loaded and merged
   - File paths resolved and validated

2. **Document Processing**
   - Markdown files parsed into AST
   - Source position tracking maintained
   - Document metadata extracted

3. **Rule Application**
   - Enabled rules identified
   - Rules applied to document AST
   - Violations collected with positions

4. **Output Generation**
   - Results formatted for display
   - Exit codes determined
   - Statistics calculated

## Rule Implementation

Rules follow a consistent pattern:

```rust
pub struct ExampleRule {
    config: ExampleConfig,
}

impl Rule for ExampleRule {
    fn id(&self) -> &'static str {
        "MD001"
    }

    fn description(&self) -> &'static str {
        "Rule description"
    }

    fn check(&self, document: &Document) -> Vec<Violation> {
        // Rule logic here
    }
}
```

### Rule Categories

- **Standard Rules** (MD001-MD059): Based on markdownlint
- **mdBook Rules** (MDBOOK001-004): mdBook-specific checks
- **Custom Rules**: Extensible for project-specific needs

## Performance Considerations

### Parsing Strategy
- Single-pass parsing per document
- AST reuse across multiple rules
- Lazy evaluation where possible

### Memory Management
- Streaming file processing for large projects
- Minimal AST retention
- Efficient string handling

### Concurrency
- Parallel file processing
- Thread-safe rule application
- Configurable worker threads

## Error Handling

### Error Categories
- **Configuration Errors**: Invalid settings, missing files
- **Parse Errors**: Malformed markdown, encoding issues
- **Rule Errors**: Internal rule failures
- **IO Errors**: File system access problems

### Error Recovery
- Graceful degradation on individual file failures
- Detailed error context and suggestions
- Configurable error tolerance levels

## Extension Points

### Custom Rules
```rust
// Plugin-style rule loading
pub fn register_custom_rule<R: Rule + 'static>(rule: R) {
    RULE_REGISTRY.register(Box::new(rule));
}
```

### Output Formats
- JSON for machine consumption
- SARIF for integration tools
- Custom formatters via traits

### Configuration Sources
- Environment variables
- External configuration services
- Runtime configuration updates

## Testing Architecture

### Test Categories
- **Unit Tests**: Individual rule logic
- **Integration Tests**: End-to-end CLI testing
- **Corpus Tests**: Real-world markdown validation
- **Performance Tests**: Benchmarking and profiling

### Test Infrastructure
- Automated test case generation
- Snapshot testing for output validation
- Property-based testing for edge cases

## Dependencies

### Core Dependencies
- `comrak`: Markdown parsing
- `serde`: Configuration serialization
- `clap`: Command-line interface
- `anyhow`: Error handling

### Development Dependencies
- `criterion`: Performance benchmarking
- `tempfile`: Test file management
- `assert_cmd`: CLI testing

## Future Architecture Considerations

### Planned Enhancements
- Plugin system for external rules
- Language server protocol support
- Real-time linting capabilities
- Integration with popular editors

### Scalability
- Distributed linting for large repositories
- Caching and incremental analysis
- Cloud-based rule execution

## Contributing to Architecture

When making architectural changes:
1. Maintain backward compatibility
2. Document design decisions
3. Consider performance implications
4. Ensure testability
5. Follow Rust best practices

## Next Steps

- See [Contributing](./contributing.md) for development guidelines
- Check [Rules Reference](./rules-reference.md) for rule implementation details
- Review source code for implementation specifics