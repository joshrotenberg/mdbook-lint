use mdbook_lint::{Document, PluginRegistry};
use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};
use std::path::PathBuf;

#[test]
fn test_duplicate_violations_md040_mdbook001() {
    let content = r#"# Test Document

This code block is missing a language tag:

```
fn main() {
    println!("Hello, world!");
}
```

This should trigger both MD040 and MDBOOK001 violations.
"#;

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();
    let violations = engine.lint_document(&document).unwrap();

    println!("Found {} violations:", violations.len());
    for (i, violation) in violations.iter().enumerate() {
        println!(
            "  {}: {} (line {}) - {}",
            i + 1,
            violation.rule_id,
            violation.line,
            violation.message
        );
    }

    // After deduplication: should only have MDBOOK001 (higher precedence)
    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        md040_violations.len(),
        0,
        "MD040 should be deduplicated in favor of MDBOOK001"
    );
    assert_eq!(
        mdbook001_violations.len(),
        1,
        "Expected exactly 1 MDBOOK001 violation"
    );

    // MDBOOK001 violation should be at line 5
    assert_eq!(mdbook001_violations[0].line, 5);
    assert!(
        mdbook001_violations[0]
            .message
            .contains("Code block is missing a language tag")
    );

    // This demonstrates successful deduplication!
    println!("DEDUPLICATION SUCCESSFUL:");
    println!("  Only MDBOOK001: {}", mdbook001_violations[0].message);
    println!("  MD040 was deduplicated (same issue, lower precedence)");
}

#[test]
fn test_multiple_code_blocks_duplicate_violations() {
    let content = r#"# Multiple Code Blocks

First block without language:

```
console.log("first");
```

Second block with language (should be fine):

```javascript
console.log("second");
```

Third block without language:

```
print("third")
```
"#;

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();
    let violations = engine.lint_document(&document).unwrap();

    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    // After deduplication: should have 0 MD040 and 2 MDBOOK001 violations
    assert_eq!(md040_violations.len(), 0, "MD040 should be deduplicated");
    assert_eq!(
        mdbook001_violations.len(),
        2,
        "Should have 2 MDBOOK001 violations (lines 5 and 17)"
    );

    // MDBOOK001 violations should be at lines 5 and 17
    assert_eq!(mdbook001_violations[0].line, 5);
    assert_eq!(mdbook001_violations[1].line, 17);

    println!(
        "Deduplication successful: {} MDBOOK001 violations, {} MD040 violations",
        mdbook001_violations.len(),
        md040_violations.len()
    );
}
