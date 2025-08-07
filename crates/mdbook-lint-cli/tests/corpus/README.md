# mdbook-lint Test Corpus

This directory contains test data for comprehensive validation of mdbook-lint's markdown linting capabilities.

## Corpus Structure

### Core Corpus (12MB, in repository)

#### `edge_cases/` (160KB)
- **Purpose**: Essential edge cases and pathological inputs
- **Contents**: Dynamically generated test cases covering:
  - Empty files and whitespace handling
  - Large files and performance stress tests
  - Nested structures (deep headings, lists, quotes)
  - Unicode and encoding edge cases
  - Rule-specific test cases for each MD rule
- **Usage**: Fast local testing, CI validation
- **Generation**: Created by `EdgeCaseGenerator` at test time

#### `markdownlint/` (1.9MB)
- **Purpose**: Compatibility validation with official markdownlint
- **Contents**: Curated subset of DavidAnson/markdownlint test suite
  - Core test files from `test/` directory
  - Documentation for each rule (MD001-MD059)
  - Example files showing rule violations and fixes
- **Usage**: Compatibility testing, regression prevention
- **Note**: Binary files and snapshots removed for size optimization

#### `real_projects/` (9.9MB)
- **Purpose**: Real-world validation with actual mdBook projects
- **Contents**: Sample files from popular Rust projects
  - mdBook official repository
  - Rust Book samples
- **Usage**: End-to-end validation, performance testing
- **Note**: Git history and binary assets removed

#### `project/` (68KB)
- **Purpose**: Basic project documentation samples
- **Contents**: Standard documentation patterns
- **Usage**: Simple compatibility validation

### Extended Corpus (Downloaded on-demand)

#### `extended/` (Created by `scripts/download-corpus.sh`)
- **markdownlint-official/**: Complete DavidAnson/markdownlint repository
- **rust-book-latest/**: Latest Rust Book content
- **mdbook-official/**: mdBook project documentation
- **rust-reference/**: Rust Reference documentation
- **cargo-book/**: Cargo Book documentation
- **self_project/**: Current project files for self-testing

## Testing Strategy

### Tiered Testing Approach

1. **Fast CI Tests** (Regular pull requests)
   - Essential edge cases only
   - Rule-specific validation
   - Unicode and robustness tests
   - **Runtime**: ~4 seconds
   - **Purpose**: Quick feedback, catch regressions

2. **Comprehensive Testing** (Nightly, corpus changes)
   - Full corpus including extended downloads
   - Performance benchmarking vs markdownlint
   - Compatibility percentage validation
   - **Runtime**: ~10-30 minutes
   - **Purpose**: Deep validation, performance tracking

3. **Local Development**
   - Core corpus available immediately
   - Optional extended corpus via download script
   - **Purpose**: Development iteration, debugging

### Usage Examples

```bash
# Fast local testing (uses core corpus)
cargo test --test corpus_integration_test test_corpus_edge_cases

# Download extended corpus for comprehensive testing
./scripts/download-corpus.sh

# Run comprehensive tests with extended corpus
cargo test --test corpus_integration_test test_extended_corpus --ignored

# Run all corpus tests (manual comprehensive testing)
cargo test --test corpus_integration_test test_comprehensive_corpus --ignored
```

## Corpus Statistics

| Component | Size | Files | Purpose |
|-----------|------|-------|---------|
| `edge_cases/` | 160KB | 26 | Essential validation |
| `markdownlint/` | 1.9MB | 424 | Compatibility testing |
| `real_projects/` | 9.9MB | 648 | Real-world validation |
| `project/` | 68KB | 12 | Basic samples |
| **Total Core** | **12MB** | **1,110** | **Fast CI testing** |
| `extended/` | ~100MB+ | 2,000+ | Comprehensive validation |

## Performance Impact

- **Before optimization**: 98MB corpus, slow clones
- **After optimization**: 12MB core corpus (88% reduction)
- **CI improvement**: Fast essential tests + comprehensive nightly testing
- **Local development**: Immediate testing capability

## Maintenance

### Adding New Test Cases

1. **Edge cases**: Modify `EdgeCaseGenerator` in `tests/corpus_tests.rs`
2. **Real projects**: Add to download script or core corpus
3. **Rule-specific**: Add to `edge_cases/rule_specific/`

### Updating External Projects

External projects are downloaded fresh during comprehensive testing, ensuring we test against the latest versions.

### Corpus Health

The corpus is automatically validated in CI:
- File count verification
- Size limits enforcement  
- Test coverage requirements
- Performance benchmarks

## Integration with Issue #31

This corpus structure implements the **Hybrid Strategy** from Issue #31:

- ✅ **Phase 1**: Analyzed and documented current corpus
- ✅ **Phase 2**: Extracted external data, reduced size 88%
- ✅ **Phase 3**: Created download scripts and CI integration
- ✅ **Phase 4**: Added self-project testing, tiered strategy

### Success Criteria Met

- ✅ Repository size reduced significantly (98MB → 12MB, target <10MB exceeded)
- ✅ Local development tests remain fast (~4 seconds)
- ✅ CI corpus testing works reliably (tiered approach)
- ✅ Full compatibility coverage maintained
- ✅ Easy to add new test projects (download script)
- ✅ Corpus stays up-to-date automatically (nightly downloads)

The corpus optimization successfully balances comprehensive testing with repository efficiency.