# Performance Profiling Guide

This guide documents techniques for identifying and debugging performance issues in mdbook-lint rules.

## Quick Performance Testing with CLI Flags

Use the new `--disable` and `--enable` flags for rapid rule testing:

```bash
# Test individual problematic rules
./target/release/mdbook-lint lint --enable MD051 large-file.md
./target/release/mdbook-lint lint --disable MD051,MD049 *.md

# Compare performance with different rule sets
time ./target/release/mdbook-lint lint --standard-only *.md
time ./target/release/mdbook-lint lint --mdbook-only *.md
```

## Systematic Performance Investigation

### 1. Build Release Binary

```bash
cargo build --release
```

### 2. Test with Increasing File Counts

Find the degradation point:

```bash
./target/release/mdbook-lint lint file1.md  # baseline
./target/release/mdbook-lint lint file{1..10}.md
./target/release/mdbook-lint lint file{1..20}.md  # usually where issues start
```

### 3. Use Divide-and-Conquer with Rule Disabling

```bash
# Test chunks of rules to isolate the problem
./target/release/mdbook-lint lint --disable MD045,MD046,MD047,MD048,MD049,MD050,MD051 *.md
./target/release/mdbook-lint lint --disable MD001,MD002,MD003,MD004,MD005 *.md
```

## CPU Profiling

### macOS (using sample)

```bash
# Profile a hanging/slow process
./target/release/mdbook-lint lint problematic-file.md &
PID=$!
sleep 3
sample $PID 5 -file profile.txt
kill $PID

# Look for high sample counts in call stack
head -100 profile.txt
```

### Linux (using perf)

```bash
# Profile with perf
perf record ./target/release/mdbook-lint lint *.md
perf report
```

Look for patterns in the call stack:

- High sample counts in specific functions
- String pattern matching functions (`TwoWaySearcher::next`, `StrSearcher::new`)
- Regex compilation in loops
- O(n²) substring operations

## Memory Profiling

### Basic Memory Usage (macOS)

```bash
/usr/bin/time -l ./target/release/mdbook-lint lint *.md 2>&1 | grep "maximum resident"
```

### Memory Leak Detection

```bash
# Test for memory leaks with increasing file counts
for i in 5 10 15 20; do
    echo "Testing $i files:"
    /usr/bin/time -l ./target/release/mdbook-lint lint $(ls *.md | head -$i) 2>&1 | grep "maximum resident"
done
```

## Known Performance Anti-patterns

Based on real issues found in mdbook-lint:

### 1. O(n²) Substring Searching (MD051 case)

```rust
// Bad: repeated substring searches create O(n²) complexity
let mut pos = 0;
while let Some(id_pos) = html[pos..].find("pattern") {
    pos += id_pos + 1; // Each find() searches remaining string
}

// Good: single-pass with iterator or regex
let regex = Regex::new(r"pattern").unwrap();
for match in regex.find_iter(html) {
    // Process match
}
```

### 2. Infinite Loops in Pattern Matching (MD049 case)  

```rust
// Bad: no bounds checking, can get stuck
while i < chars.len() {
    if chars[i] == target {
        // Complex logic that might not advance i
        if some_condition {
            continue; // Dangerous - infinite loop if condition always true
        }
    }
    i += 1; // Make sure this always executes
}

// Good: explicit bounds checking and advancement
while i < chars.len() {
    let ch = chars[i];
    i += 1; // Always advance first
    
    if ch == target {
        // Process logic
    }
}
```

### 3. Regex Compilation in Loops

```rust
// Bad: compiles regex repeatedly
for line in lines {
    let re = Regex::new(pattern).unwrap(); // Expensive compilation
    if re.is_match(line) { }
}

// Good: compile once, use many times  
let re = Regex::new(pattern).unwrap();
for line in lines {
    if re.is_match(line) { }
}
```

## Performance Testing Corpus

### Recommended Test Files

- **The Rust Book**: 112 markdown files, excellent real-world test

  ```bash

  git clone --depth 1 https://github.com/rust-lang/book.git /tmp/rust-book
  ./target/release/mdbook-lint lint /tmp/rust-book/src/*.md
  ```

### Performance Benchmarks

- **Expected performance**: <1 second for 100+ files with well-behaved rules
- **Red flags**: >5 seconds or timeouts indicate algorithmic issues
- **Memory usage**: Should stay under 50MB for large document sets

### Specific Problematic Files

Some files are known to trigger performance issues:

- Files with many HTML snippets (triggers MD051)
- Files with mixed emphasis markers like `wrapping_*` (triggers MD049)
- Very large files (>1000 lines)

## Debugging Rule-Specific Issues

### Test Single Rule in Isolation

```bash
# Method 1: Using CLI flags (recommended)
./target/release/mdbook-lint lint --enable MD051 problematic-file.md

# Method 2: Using configuration file
echo "[rules]
default = false
[rules.enabled]  
MD051 = true" > test-config.toml

./target/release/mdbook-lint lint -c test-config.toml problematic-file.md
```

### Performance Regression Testing

```bash
# Create a simple benchmark
echo "#!/bin/bash
time ./target/release/mdbook-lint lint /tmp/rust-book/src/*.md" > benchmark.sh
chmod +x benchmark.sh

# Run before and after changes
git checkout main && ./benchmark.sh
git checkout feature-branch && ./benchmark.sh
```

## Real-World Examples

### Case Study: MD051 O(n²) Issue

**Problem**: MD051 hung indefinitely on The Rust Book
**Diagnosis**: Profiling showed 80% CPU time in `TwoWaySearcher::next`
**Solution**: Replace substring searching with regex iterator
**Result**: 112 files processed in 0.32 seconds instead of timeout

### Case Study: MD049 Infinite Loop

**Problem**: Mixed emphasis markers caused hangs
**Diagnosis**: Pattern `wrapping_*` confused emphasis detection
**Solution**: Better shell prompt detection and bounds checking
**Result**: Rule now handles edge cases correctly

## Tools and Resources

### Profiling Tools

- **macOS**: `sample`, `instruments`, `/usr/bin/time -l`  
- **Linux**: `perf`, `valgrind`, `time`
- **Cross-platform**: `cargo flamegraph`, `cargo bench`

### Useful Links

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [mdbook-lint Performance Issues](https://github.com/joshrotenberg/mdbook-lint/issues?q=label%3Aperformance)

## Contributing Performance Fixes

When fixing performance issues:

1. **Document the problem**: Create detailed issue with profiling data
2. **Add benchmarks**: Include before/after performance measurements  
3. **Test edge cases**: Ensure fix doesn't break functionality
4. **Update this guide**: Document new anti-patterns or techniques discovered
