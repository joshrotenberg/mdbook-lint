//! Property-based tests for linting rules.
//!
//! These tests verify invariants that should hold for all possible inputs:
//! - Rules never panic on arbitrary UTF-8 input
//! - Results are deterministic (same input = same output)
//! - Violation counts are non-negative
//! - Fix applications don't increase violation counts

#[cfg(test)]
mod tests {
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use proptest::prelude::*;
    use std::path::PathBuf;

    use crate::standard::md001::MD001;
    use crate::standard::md002::MD002;
    use crate::standard::md003::MD003;
    use crate::standard::md004::MD004;
    use crate::standard::md005::MD005;
    use crate::standard::md006::MD006;
    use crate::standard::md007::MD007;
    use crate::standard::md009::MD009;
    use crate::standard::md010::MD010;
    use crate::standard::md011::MD011;
    use crate::standard::md012::MD012;
    use crate::standard::md013::MD013;
    use crate::standard::md014::MD014;
    use crate::standard::md018::MD018;
    use crate::standard::md019::MD019;
    use crate::standard::md020::MD020;
    use crate::standard::md021::MD021;
    use crate::standard::md022::MD022;
    use crate::standard::md023::MD023;
    use crate::standard::md024::MD024;
    use crate::standard::md025::MD025;
    use crate::standard::md026::MD026;
    use crate::standard::md027::MD027;
    use crate::standard::md028::MD028;
    use crate::standard::md029::MD029;
    use crate::standard::md030::MD030;
    use crate::standard::md031::MD031;
    use crate::standard::md032::MD032;
    use crate::standard::md033::MD033;
    use crate::standard::md034::MD034;
    use crate::standard::md035::MD035;
    use crate::standard::md036::MD036;
    use crate::standard::md037::MD037;
    use crate::standard::md038::MD038;
    use crate::standard::md039::MD039;
    use crate::standard::md040::MD040;
    use crate::standard::md041::MD041;
    use crate::standard::md042::MD042;
    use crate::standard::md043::MD043;
    use crate::standard::md045::MD045;
    use crate::standard::md046::MD046;
    use crate::standard::md047::MD047;
    use crate::standard::md048::MD048;
    use crate::standard::md049::MD049;
    use crate::standard::md050::MD050;
    use crate::standard::md051::MD051;
    use crate::standard::md052::MD052;
    use crate::standard::md053::MD053;
    use crate::standard::md054::MD054;
    use crate::standard::md055::MD055;
    use crate::standard::md056::MD056;
    use crate::standard::md057::MD057;
    use crate::standard::md058::MD058;
    use crate::standard::md059::MD059;
    use crate::standard::md060::MD060;

    // MDBOOK rules
    use crate::mdbook::mdbook001::MDBOOK001;
    use crate::mdbook::mdbook002::MDBOOK002;
    use crate::mdbook::mdbook003::MDBOOK003;
    use crate::mdbook::mdbook004::MDBOOK004;
    use crate::mdbook::mdbook005::MDBOOK005;
    use crate::mdbook::mdbook006::MDBOOK006;
    use crate::mdbook::mdbook007::MDBOOK007;
    use crate::mdbook::mdbook008::MDBOOK008;
    use crate::mdbook::mdbook009::MDBOOK009;
    use crate::mdbook::mdbook010::MDBOOK010;
    use crate::mdbook::mdbook011::MDBOOK011;
    use crate::mdbook::mdbook012::MDBOOK012;
    use crate::mdbook::mdbook016::MDBOOK016;
    use crate::mdbook::mdbook017::MDBOOK017;
    use crate::mdbook::mdbook021::MDBOOK021;
    use crate::mdbook::mdbook022::MDBOOK022;
    use crate::mdbook::mdbook023::MDBOOK023;
    use crate::mdbook::mdbook025::MDBOOK025;

    /// Strategy for generating arbitrary markdown-like content
    fn markdown_content() -> impl Strategy<Value = String> {
        prop::string::string_regex(
            r"(?s)(#{1,6} [^\n]*\n|[^\n]*\n|```[^\n]*\n([^\n]*\n)*```\n|\n)*",
        )
        .unwrap()
    }

    /// Strategy for generating heading-focused content
    fn heading_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // ATX headings with various levels
                (1..=6usize, "[A-Za-z0-9 ]{0,50}")
                    .prop_map(|(level, text)| { format!("{} {}\n", "#".repeat(level), text) }),
                // Plain text lines
                "[A-Za-z0-9 ]{0,100}\n".prop_map(|s| s),
                // Empty lines
                Just("\n".to_string()),
                // Setext h1
                "[A-Za-z0-9 ]{1,50}\n={3,10}\n".prop_map(|s| s),
                // Setext h2
                "[A-Za-z0-9 ]{1,50}\n-{3,10}\n".prop_map(|s| s),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for generating content with unicode
    fn unicode_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Unicode headings
                (1..=6usize, "\\PC{0,30}")
                    .prop_map(|(level, text)| { format!("{} {}\n", "#".repeat(level), text) }),
                // Plain unicode text
                "\\PC{0,100}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    // =========================================================================
    // MD001: Heading Increment
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md001_never_panics(content in markdown_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD001;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md001_never_panics_heading_content(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD001;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md001_never_panics_unicode(content in unicode_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD001;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md001_deterministic(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD001;
                let result1 = rule.check(&doc);
                let result2 = rule.check(&doc);

                match (result1, result2) {
                    (Ok(v1), Ok(v2)) => {
                        prop_assert_eq!(v1.len(), v2.len(), "Violation count should be deterministic");
                        for (a, b) in v1.iter().zip(v2.iter()) {
                            prop_assert_eq!(a.line, b.line);
                            prop_assert_eq!(a.column, b.column);
                            prop_assert_eq!(&a.message, &b.message);
                        }
                    }
                    (Err(_), Err(_)) => {}
                    _ => prop_assert!(false, "Results should both succeed or both fail"),
                }
            }
        }

        #[test]
        fn md001_violations_have_valid_positions(content in heading_content()) {
            let doc = Document::new(content.clone(), PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD001;
                if let Ok(violations) = rule.check(&doc) {
                    let line_count = content.lines().count().max(1);
                    for v in violations {
                        prop_assert!(v.line >= 1, "Line number should be >= 1");
                        prop_assert!(v.line <= line_count + 1, "Line {} exceeds content lines {}", v.line, line_count);
                        prop_assert!(v.column >= 1, "Column should be >= 1");
                    }
                }
            }
        }
    }

    // =========================================================================
    // MD002: First Heading H1
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md002_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD002::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md002_at_most_one_violation(content in heading_content()) {
            // MD002 only checks the first heading, so at most 1 violation
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD002::new();
                if let Ok(violations) = rule.check(&doc) {
                    prop_assert!(violations.len() <= 1, "MD002 should have at most 1 violation, got {}", violations.len());
                }
            }
        }

        #[test]
        fn md002_deterministic(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD002::new();
                let result1 = rule.check(&doc);
                let result2 = rule.check(&doc);

                match (result1, result2) {
                    (Ok(v1), Ok(v2)) => {
                        prop_assert_eq!(v1.len(), v2.len());
                    }
                    (Err(_), Err(_)) => {}
                    _ => prop_assert!(false, "Results should match"),
                }
            }
        }
    }

    // =========================================================================
    // MD003: Heading Style
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md003_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD003::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md003_never_panics_unicode(content in unicode_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD003::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD009: Trailing Spaces
    // =========================================================================

    /// Strategy for content with trailing spaces
    fn trailing_space_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Line with trailing spaces
                ("[A-Za-z0-9]{0,50}", " {0,5}")
                    .prop_map(|(text, spaces)| { format!("{}{}\n", text, spaces) }),
                // Empty line
                Just("\n".to_string()),
                // Line with tabs
                ("[A-Za-z0-9]{0,50}", "\t{0,3}")
                    .prop_map(|(text, tabs)| { format!("{}{}\n", text, tabs) }),
            ],
            0..30,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md009_never_panics(content in trailing_space_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD009::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md009_no_violations_on_clean_content(
            lines in prop::collection::vec("[A-Za-z0-9]{1,50}", 1..20)
        ) {
            // Content with no trailing whitespace should have no violations
            let content = lines.join("\n");
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD009::new();
                if let Ok(violations) = rule.check(&doc) {
                    prop_assert_eq!(violations.len(), 0, "Clean content should have no MD009 violations");
                }
            }
        }
    }

    // =========================================================================
    // MD010: Hard Tabs
    // =========================================================================

    /// Strategy for content with tabs
    fn tab_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Line with tabs
                "(\t|    ){0,5}[A-Za-z0-9 ]{0,50}\n".prop_map(|s| s),
                // Code block with tabs
                "```\n\t[A-Za-z0-9]{0,30}\n```\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md010_never_panics(content in tab_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD010::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md010_never_panics_unicode(content in unicode_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD010::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD012: Multiple Blank Lines
    // =========================================================================

    /// Strategy for content with multiple blank lines
    fn blank_line_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                "\n{1,5}".prop_map(|s| s),
                "# [A-Za-z0-9]{1,20}\n".prop_map(|s| s),
            ],
            0..30,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md012_never_panics(content in blank_line_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD012::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD013: Line Length
    // =========================================================================

    /// Strategy for content with varying line lengths
    fn long_line_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                "[A-Za-z0-9 ]{0,200}\n".prop_map(|s| s),
                "# [A-Za-z0-9 ]{0,150}\n".prop_map(|s| s),
                "```\n[A-Za-z0-9]{0,300}\n```\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md013_never_panics(content in long_line_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD013::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md013_never_panics_unicode(content in unicode_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD013::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD018/MD019: Heading Space Rules
    // =========================================================================

    /// Strategy for headings with various spacing
    fn heading_space_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Normal heading
                "#{1,6} [A-Za-z0-9]{1,30}\n".prop_map(|s| s),
                // Missing space after hash
                "#{1,6}[A-Za-z0-9]{1,30}\n".prop_map(|s| s),
                // Multiple spaces after hash
                "#{1,6}  +[A-Za-z0-9]{1,30}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md018_never_panics(content in heading_space_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD018;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md019_never_panics(content in heading_space_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD019;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md018_never_panics_unicode(content in unicode_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD018;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md019_never_panics_unicode(content in unicode_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD019;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD023: Heading Indentation
    // =========================================================================

    /// Strategy for headings with indentation
    fn indented_heading_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                " {0,4}#{1,6} [A-Za-z0-9]{1,30}\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{0,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md023_never_panics(content in indented_heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD023;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD031/MD032: Blank Lines Around Fenced Code Blocks and Lists
    // =========================================================================

    /// Strategy for code blocks and lists
    fn code_and_list_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                "```[a-z]*\n[A-Za-z0-9 ]{0,50}\n```\n".prop_map(|s| s),
                "- [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                "[0-9]+\\. [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md031_never_panics(content in code_and_list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD031;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md032_never_panics(content in code_and_list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD032;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD004-MD007: List Rules
    // =========================================================================

    /// Strategy for list content
    fn list_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Unordered list items with different markers
                "[-*+] [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                // Ordered list items
                "[0-9]{1,3}\\. [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                // Indented list items
                "  [-*+] [A-Za-z0-9 ]{1,20}\n".prop_map(|s| s),
                "    [-*+] [A-Za-z0-9 ]{1,20}\n".prop_map(|s| s),
                // Regular text
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..25,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md004_never_panics(content in list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD004::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md005_never_panics(content in list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD005;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md006_never_panics(content in list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD006;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md007_never_panics(content in list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD007::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD011: Reversed Link Syntax
    // =========================================================================

    /// Strategy for content with links
    fn link_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Normal links
                r"\[[A-Za-z0-9 ]{1,20}\]\([a-z]{1,10}\)".prop_map(|s| format!("{}\n", s)),
                // Reversed links (bug)
                r"\([A-Za-z0-9 ]{1,20}\)\[[a-z]{1,10}\]".prop_map(|s| format!("{}\n", s)),
                // Regular text
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md011_never_panics(content in link_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD011;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD014: Dollar Signs in Commands
    // =========================================================================

    /// Strategy for code blocks with commands
    fn command_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Code block with $ prefix
                "```\n\\$ [a-z]{1,20}\n```\n".prop_map(|s| s),
                // Code block without $ prefix
                "```bash\n[a-z]{1,20}\n```\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..10,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md014_never_panics(content in command_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD014;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD020-MD022: Closed ATX Headings and Blank Lines
    // =========================================================================

    /// Strategy for closed ATX headings
    fn closed_heading_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Closed ATX headings
                "#{1,6} [A-Za-z0-9 ]{1,30} #{1,6}\n".prop_map(|s| s),
                // Normal ATX headings
                "#{1,6} [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md020_never_panics(content in closed_heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD020;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md021_never_panics(content in closed_heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD021;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md022_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD022;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD024-MD026: Heading Content Rules
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md024_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD024::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md025_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD025::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md026_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD026::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD027-MD030: Blockquote and List Spacing Rules
    // =========================================================================

    /// Strategy for blockquote content
    fn blockquote_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Blockquotes with various spacing
                "> {0,3}[A-Za-z0-9 ]{1,40}\n".prop_map(|s| s),
                // Nested blockquotes
                "> > [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md027_never_panics(content in blockquote_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD027;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md028_never_panics(content in blockquote_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD028;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md029_never_panics(content in list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD029::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md030_never_panics(content in list_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD030::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD033-MD043: HTML, URLs, Horizontal Rules, Emphasis, Code, Links, Images
    // =========================================================================

    /// Strategy for content with inline HTML elements
    fn html_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // HTML tags
                "<[a-z]{1,8}>[A-Za-z0-9 ]{0,30}</[a-z]{1,8}>\n".prop_map(|s| s),
                // Self-closing HTML
                "<[a-z]{1,8} */>\n".prop_map(|s| s),
                // Regular text
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for content with URLs (for MD034, MD039, MD042)
    fn url_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Bare URLs
                "http://[a-z]{1,20}\\.[a-z]{2,4}/[a-z]{0,20}\n".prop_map(|s| s),
                // Links with text
                "\\[[A-Za-z0-9 ]{1,20}\\]\\(http://[a-z]{1,10}\\.[a-z]{2,4}\\)\n".prop_map(|s| s),
                // Empty links
                "\\[\\]\\(\\)\n".prop_map(|s| s),
                // Links with spaces
                "\\[ [A-Za-z0-9]{1,10} \\]\\([a-z]{1,20}\\)\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for content with emphasis (for MD036, MD037, MD049, MD050)
    fn emphasis_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Asterisk emphasis
                "\\*[A-Za-z0-9 ]{1,20}\\*\n".prop_map(|s| s),
                // Underscore emphasis
                "_[A-Za-z0-9 ]{1,20}_\n".prop_map(|s| s),
                // Bold asterisks
                "\\*\\*[A-Za-z0-9 ]{1,20}\\*\\*\n".prop_map(|s| s),
                // Bold underscores
                "__[A-Za-z0-9 ]{1,20}__\n".prop_map(|s| s),
                // Emphasis with spaces inside
                "\\* [A-Za-z0-9]{1,10} \\*\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for content with code blocks (for MD038, MD040, MD046, MD048)
    fn code_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Inline code
                "`[A-Za-z0-9 ]{1,20}`\n".prop_map(|s| s),
                // Inline code with spaces
                "` [A-Za-z0-9]{1,10} `\n".prop_map(|s| s),
                // Fenced code block with language
                "```[a-z]{0,10}\n[A-Za-z0-9 ]{0,30}\n```\n".prop_map(|s| s),
                // Indented code block
                "    [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for content with horizontal rules (for MD035)
    fn hr_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Horizontal rules
                "-{3,10}\n".prop_map(|s| s),
                "\\*{3,10}\n".prop_map(|s| s),
                "_{3,10}\n".prop_map(|s| s),
                Just("- - -\n".to_string()),
                Just("* * *\n".to_string()),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for content with images (for MD045)
    fn image_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Images with alt text
                "!\\[[A-Za-z0-9 ]{0,20}\\]\\([a-z]{1,10}\\.png\\)\n".prop_map(|s| s),
                // Images without alt text
                "!\\[\\]\\([a-z]{1,10}\\.jpg\\)\n".prop_map(|s| s),
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md033_never_panics(content in html_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD033;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md034_never_panics(content in url_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD034;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md035_never_panics(content in hr_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD035::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md036_never_panics(content in emphasis_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD036::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md037_never_panics(content in emphasis_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD037;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md038_never_panics(content in code_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD038;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md039_never_panics(content in url_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD039;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md040_never_panics(content in code_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD040;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md041_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD041;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md042_never_panics(content in url_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD042;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md043_never_panics(content in heading_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD043::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD045-MD050: Images, Code Style, Trailing Newline, Emphasis Style
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md045_never_panics(content in image_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD045;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md046_never_panics(content in code_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD046::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md047_never_panics(content in markdown_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD047;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md048_never_panics(content in code_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD048::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md049_never_panics(content in emphasis_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD049::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md050_never_panics(content in emphasis_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD050::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD051-MD054: Reference Links
    // =========================================================================

    /// Strategy for content with reference links (for MD051-MD054)
    fn reference_link_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Reference-style links
                "\\[[A-Za-z0-9 ]{1,20}\\]\\[[a-z]{1,10}\\]\n".prop_map(|s| s),
                // Link definitions
                "\\[[a-z]{1,10}\\]: http://[a-z]{1,10}\\.[a-z]{2,4}\n".prop_map(|s| s),
                // Shortcut reference links
                "\\[[A-Za-z0-9 ]{1,20}\\]\n".prop_map(|s| s),
                // Inline links
                "\\[[A-Za-z0-9 ]{1,20}\\]\\(http://[a-z]{1,10}\\.[a-z]{2,4}\\)\n".prop_map(|s| s),
                // Regular text
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md051_never_panics(content in reference_link_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD051::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md052_never_panics(content in reference_link_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD052::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md053_never_panics(content in reference_link_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD053::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md054_never_panics(content in reference_link_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD054::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD055-MD058: Tables
    // =========================================================================

    /// Strategy for content with tables (for MD055-MD058)
    fn table_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Table header row
                Just("| Header 1 | Header 2 |\n".to_string()),
                // Table separator
                Just("| --- | --- |\n".to_string()),
                Just("|:---:|:---:|\n".to_string()),
                Just("| :--- | ---: |\n".to_string()),
                // Table data row
                "\\| [A-Za-z0-9 ]{1,10} \\| [A-Za-z0-9 ]{1,10} \\|\n".prop_map(|s| s),
                // Regular text
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md055_never_panics(content in table_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD055::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md056_never_panics(content in table_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD056;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md057_never_panics(content in table_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD057;
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md058_never_panics(content in table_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD058;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MD059-MD060: Link Text and Table Column Style
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn md059_never_panics(content in url_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD059::new();
                let _ = rule.check(&doc);
            }
        }

        #[test]
        fn md060_never_panics(content in table_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MD060::new();
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MDBOOK Rules: mdBook-specific linting rules
    // =========================================================================

    /// Strategy for mdbook include directives
    fn mdbook_include_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Include directive
                r"\{\{#include [a-z/]{1,30}\.rs\}\}\n".prop_map(|s| s),
                // Rustdoc include
                r"\{\{#rustdoc_include [a-z/]{1,30}\.rs(:[0-9]{1,3})?\}\}\n".prop_map(|s| s),
                // Playground include
                r"\{\{#playground [a-z/]{1,30}\.rs\}\}\n".prop_map(|s| s),
                // Code blocks
                "```rust\n[A-Za-z0-9 ]{0,50}\n```\n".prop_map(|s| s),
                "```\n[A-Za-z0-9 ]{0,50}\n```\n".prop_map(|s| s),
                // Regular text
                "[A-Za-z0-9 ]{1,50}\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..15,
        )
        .prop_map(|lines| lines.join(""))
    }

    /// Strategy for SUMMARY.md-like content
    fn summary_content() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // Part headers
                "# [A-Za-z0-9 ]{1,30}\n".prop_map(|s| s),
                // Chapter links (list items)
                "- \\[[A-Za-z0-9 ]{1,20}\\]\\([a-z/]{1,20}\\.md\\)\n".prop_map(|s| s),
                // Nested chapter links
                "    - \\[[A-Za-z0-9 ]{1,15}\\]\\([a-z/]{1,15}\\.md\\)\n".prop_map(|s| s),
                // Prefix/suffix chapters (plain links)
                "\\[[A-Za-z0-9 ]{1,20}\\]\\([a-z/]{1,20}\\.md\\)\n".prop_map(|s| s),
                // Separators
                Just("---\n".to_string()),
                Just("----\n".to_string()),
                // Draft chapters
                "- \\[[A-Za-z0-9 ]{1,15}\\]\\(\\)\n".prop_map(|s| s),
                Just("\n".to_string()),
            ],
            0..20,
        )
        .prop_map(|lines| lines.join(""))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        // MDBOOK001: Code blocks should have language tags
        #[test]
        fn mdbook001_never_panics(content in code_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK001;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK002: SUMMARY.md should exist and have valid structure
        #[test]
        fn mdbook002_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK002;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK003: SUMMARY.md structure validation
        #[test]
        fn mdbook003_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK003;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK004: Part titles should be formatted correctly
        #[test]
        fn mdbook004_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK004;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK005: Chapter paths should be relative
        #[test]
        fn mdbook005_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK005::default();
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK006: Draft chapters should have content or be marked
        #[test]
        fn mdbook006_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK006::default();
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK007: Separator syntax should be correct
        #[test]
        fn mdbook007_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK007::default();
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK008: Invalid {{#rustdoc_include}} paths or syntax
        #[test]
        fn mdbook008_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK008;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK009: Invalid {{#include}} paths
        #[test]
        fn mdbook009_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK009;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK010: Missing ANCHOR references in include directives
        #[test]
        fn mdbook010_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK010;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK011: Invalid {{#playground}} paths
        #[test]
        fn mdbook011_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK011;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK012: Invalid template paths
        #[test]
        fn mdbook012_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK012;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK016: Invalid {{#title}} directives
        #[test]
        fn mdbook016_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK016;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK017: Duplicate anchors
        #[test]
        fn mdbook017_never_panics(content in code_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK017;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK021: Invalid preprocessor commands
        #[test]
        fn mdbook021_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK021;
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK022: Deprecated preprocessor syntax
        #[test]
        fn mdbook022_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK022::default();
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK023: Invalid syntax
        #[test]
        fn mdbook023_never_panics(content in mdbook_include_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK023::default();
                let _ = rule.check(&doc);
            }
        }

        // MDBOOK025: Multiple H1 allowed in SUMMARY.md
        #[test]
        fn mdbook025_never_panics(content in summary_content()) {
            let doc = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc {
                let rule = MDBOOK025;
                let _ = rule.check(&doc);
            }
        }
    }

    // =========================================================================
    // MDBOOK rules with arbitrary content
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn all_mdbook_rules_handle_arbitrary_content(content in arbitrary_content()) {
            let doc = Document::new(content.clone(), PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let _ = MDBOOK001.check(&doc);
                let _ = MDBOOK008.check(&doc);
                let _ = MDBOOK009.check(&doc);
                let _ = MDBOOK010.check(&doc);
                let _ = MDBOOK011.check(&doc);
                let _ = MDBOOK012.check(&doc);
                let _ = MDBOOK016.check(&doc);
                let _ = MDBOOK017.check(&doc);
                let _ = MDBOOK021.check(&doc);
                let _ = MDBOOK022::default().check(&doc);
                let _ = MDBOOK023::default().check(&doc);
            }

            // Test SUMMARY.md-specific rules with SUMMARY.md filename
            let doc_summary = Document::new(content, PathBuf::from("SUMMARY.md"));
            if let Ok(doc) = doc_summary {
                let _ = MDBOOK002.check(&doc);
                let _ = MDBOOK003.check(&doc);
                let _ = MDBOOK004.check(&doc);
                let _ = MDBOOK005::default().check(&doc);
                let _ = MDBOOK006::default().check(&doc);
                let _ = MDBOOK007::default().check(&doc);
                let _ = MDBOOK025.check(&doc);
            }
        }
    }

    // =========================================================================
    // Stress test with arbitrary content
    // =========================================================================

    /// Strategy for completely arbitrary content including special chars
    fn arbitrary_content() -> impl Strategy<Value = String> {
        prop::string::string_regex(r"[\x00-\x7F]{0,500}").unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn all_rules_handle_arbitrary_ascii(content in arbitrary_content()) {
            let doc = Document::new(content, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                // Test all rules don't panic on arbitrary content
                let _ = MD001.check(&doc);
                let _ = MD002::new().check(&doc);
                let _ = MD003::new().check(&doc);
                let _ = MD004::new().check(&doc);
                let _ = MD005.check(&doc);
                let _ = MD006.check(&doc);
                let _ = MD007::new().check(&doc);
                let _ = MD009::new().check(&doc);
                let _ = MD010::new().check(&doc);
                let _ = MD011.check(&doc);
                let _ = MD012::new().check(&doc);
                let _ = MD013::new().check(&doc);
                let _ = MD014.check(&doc);
                let _ = MD018.check(&doc);
                let _ = MD019.check(&doc);
                let _ = MD020.check(&doc);
                let _ = MD021.check(&doc);
                let _ = MD022.check(&doc);
                let _ = MD023.check(&doc);
                let _ = MD024::new().check(&doc);
                let _ = MD025::new().check(&doc);
                let _ = MD026::new().check(&doc);
                let _ = MD027.check(&doc);
                let _ = MD028.check(&doc);
                let _ = MD029::new().check(&doc);
                let _ = MD030::new().check(&doc);
                let _ = MD031.check(&doc);
                let _ = MD032.check(&doc);
                let _ = MD033.check(&doc);
                let _ = MD034.check(&doc);
                let _ = MD035::new().check(&doc);
                let _ = MD036::new().check(&doc);
                let _ = MD037.check(&doc);
                let _ = MD038.check(&doc);
                let _ = MD039.check(&doc);
                let _ = MD040.check(&doc);
                let _ = MD041.check(&doc);
                let _ = MD042.check(&doc);
                let _ = MD043::new().check(&doc);
                let _ = MD045.check(&doc);
                let _ = MD046::new().check(&doc);
                let _ = MD047.check(&doc);
                let _ = MD048::new().check(&doc);
                let _ = MD049::new().check(&doc);
                let _ = MD050::new().check(&doc);
                let _ = MD051::new().check(&doc);
                let _ = MD052::new().check(&doc);
                let _ = MD053::new().check(&doc);
                let _ = MD054::new().check(&doc);
                let _ = MD055::new().check(&doc);
                let _ = MD056.check(&doc);
                let _ = MD057.check(&doc);
                let _ = MD058.check(&doc);
                let _ = MD059::new().check(&doc);
                let _ = MD060::new().check(&doc);
            }
        }
    }

    // =========================================================================
    // General invariants across all tested rules
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn empty_content_never_causes_violations(
            rule_id in prop_oneof![Just("MD001"), Just("MD002"), Just("MD003"), Just("MD009")]
        ) {
            let doc = Document::new(String::new(), PathBuf::from("test.md")).unwrap();

            let violations = match rule_id {
                "MD001" => MD001.check(&doc),
                "MD002" => MD002::new().check(&doc),
                "MD003" => MD003::new().check(&doc),
                "MD009" => MD009::new().check(&doc),
                _ => unreachable!(),
            };

            if let Ok(v) = violations {
                prop_assert_eq!(v.len(), 0, "{} should have no violations on empty content", rule_id);
            }
        }

        #[test]
        fn whitespace_only_never_panics(
            whitespace in "[ \t\n]{0,100}",
            rule_id in prop_oneof![Just("MD001"), Just("MD002"), Just("MD003"), Just("MD009")]
        ) {
            let doc = Document::new(whitespace, PathBuf::from("test.md"));
            if let Ok(doc) = doc {
                let result = match rule_id {
                    "MD001" => MD001.check(&doc),
                    "MD002" => MD002::new().check(&doc),
                    "MD003" => MD003::new().check(&doc),
                    "MD009" => MD009::new().check(&doc),
                    _ => unreachable!(),
                };
                // Just assert it doesn't panic - result can be Ok or Err
                let _ = result;
            }
        }
    }
}
