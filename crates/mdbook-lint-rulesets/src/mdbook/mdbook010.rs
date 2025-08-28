//! MDBOOK010: Missing or invalid preprocessor configuration
//!
//! This rule checks for invalid preprocessor directives in mdBook files.
//! Preprocessors like mermaid, katex, and others require specific syntax.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;

/// Rule to check for invalid preprocessor configuration
pub struct MDBOOK010;

impl Rule for MDBOOK010 {
    fn id(&self) -> &'static str {
        "MDBOOK010"
    }

    fn name(&self) -> &'static str {
        "preprocessor-validation"
    }

    fn description(&self) -> &'static str {
        "Missing or invalid preprocessor configuration"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::experimental(RuleCategory::MdBook).introduced_in("mdbook-lint v0.11.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Check for common preprocessor patterns
        self.check_mermaid_blocks(document, &mut violations);
        self.check_katex_blocks(document, &mut violations);
        self.check_admonish_blocks(document, &mut violations);

        Ok(violations)
    }
}

impl MDBOOK010 {
    /// Check for invalid mermaid blocks
    fn check_mermaid_blocks(&self, document: &Document, violations: &mut Vec<Violation>) {
        let _mermaid_re = Regex::new(r"```mermaid\s*\n([\s\S]*?)```").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            if line.trim() == "```mermaid" {
                // Check if the block is empty
                if line_num + 1 < document.lines.len() {
                    let next_line = &document.lines[line_num + 1];
                    if next_line.trim() == "```" {
                        violations.push(self.create_violation(
                            "Empty mermaid block detected".to_string(),
                            line_num + 1,
                            1,
                            Severity::Warning,
                        ));
                    }
                }
            }

            // Check for common mermaid syntax errors
            if line.contains("```mermaid") && !line.trim().eq("```mermaid") {
                violations.push(self.create_violation(
                    "Mermaid blocks should start with '```mermaid' on its own line".to_string(),
                    line_num + 1,
                    1,
                    Severity::Error,
                ));
            }
        }
    }

    /// Check for invalid KaTeX blocks
    fn check_katex_blocks(&self, document: &Document, violations: &mut Vec<Violation>) {
        // Check for inline math
        let _inline_math_re = Regex::new(r"\$([^$\n]+)\$").unwrap();
        // Check for display math
        let _display_math_re = Regex::new(r"\$\$([^$]+)\$\$").unwrap();

        let mut in_code_block = false;

        for (line_num, line) in document.lines.iter().enumerate() {
            // Track code block state - skip all processing inside code blocks
            if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip all processing if we're inside a code block
            if in_code_block {
                continue;
            }

            // Skip lines that look like shell prompts (start with $ followed by space or common commands)
            let trimmed = line.trim();
            if trimmed.starts_with("$ ") || trimmed == "$" {
                // This looks like a shell prompt, not a math block
                continue;
            }

            // Check for unclosed inline math - but exclude standalone $ at start of line
            let dollar_count = line.chars().filter(|&c| c == '$').count();
            if dollar_count % 2 != 0 && !line.contains("$$") {
                // Additional check: only flag if there's actual math content
                // A single $ at the start followed by non-math content is likely a shell prompt
                if !Self::is_likely_shell_prompt(line) {
                    violations.push(self.create_violation(
                        "Unclosed inline math block (odd number of $ signs)".to_string(),
                        line_num + 1,
                        1,
                        Severity::Error,
                    ));
                }
            }

            // Check for empty math blocks (still only outside code blocks)
            if line.contains("$$$$") {
                violations.push(self.create_violation(
                    "Empty display math block detected".to_string(),
                    line_num + 1,
                    line.find("$$$$").unwrap() + 1,
                    Severity::Warning,
                ));
            }

            if line.contains("$ $") {
                violations.push(self.create_violation(
                    "Empty inline math block detected".to_string(),
                    line_num + 1,
                    line.find("$ $").unwrap() + 1,
                    Severity::Warning,
                ));
            }
        }
    }

    /// Helper function to detect if a line is likely a shell prompt
    fn is_likely_shell_prompt(line: &str) -> bool {
        let trimmed = line.trim();

        // Common shell prompt patterns
        if trimmed.starts_with("$ ") || trimmed == "$" {
            return true;
        }

        // Check for PowerShell prompts
        if trimmed.starts_with("> ") || trimmed == ">" {
            return true;
        }

        // Check for numbered prompts like "1$" or similar
        if trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains("$") {
            return true;
        }

        // If line has a single $ and looks like a command (contains common shell keywords)
        if line.matches('$').count() == 1 {
            let after_dollar = line.split('$').nth(1).unwrap_or("");
            let first_word = after_dollar.split_whitespace().next().unwrap_or("");

            // Common shell commands/keywords
            let shell_keywords = [
                "cd", "ls", "pwd", "echo", "mkdir", "rm", "cp", "mv", "cat", "grep", "find",
                "curl", "wget", "git", "npm", "cargo", "rustc", "python", "ruby", "node", "java",
                "gcc", "make", "sudo", "apt", "yum", "brew", "docker", "kubectl", "rustup", "pip",
                "gem", "yarn", "pnpm", "deno", "./", "../", "~/", "/", "\\", ".", "..", "export",
                "source", "bash",
            ];

            if shell_keywords.iter().any(|&kw| first_word.starts_with(kw)) {
                return true;
            }
        }

        false
    }

    /// Check for invalid admonish blocks
    fn check_admonish_blocks(&self, document: &Document, violations: &mut Vec<Violation>) {
        let admonish_re = Regex::new(r"```admonish\s+(\w+)(.*)").unwrap();
        let valid_types = [
            "note",
            "tip",
            "info",
            "warning",
            "danger",
            "important",
            "caution",
            "bug",
            "example",
            "quote",
        ];

        for (line_num, line) in document.lines.iter().enumerate() {
            if line.starts_with("```admonish") {
                if let Some(captures) = admonish_re.captures(line) {
                    if let Some(admonish_type) = captures.get(1) {
                        let type_str = admonish_type.as_str();
                        if !valid_types.contains(&type_str) {
                            violations.push(self.create_violation(
                                format!(
                                    "Invalid admonish type '{}'. Valid types are: {}",
                                    type_str,
                                    valid_types.join(", ")
                                ),
                                line_num + 1,
                                admonish_type.start() + 1,
                                Severity::Error,
                            ));
                        }
                    }
                } else if line.trim() == "```admonish" {
                    violations.push(self.create_violation(
                        "Admonish block missing type. Use format: ```admonish <type>".to_string(),
                        line_num + 1,
                        1,
                        Severity::Error,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use std::path::PathBuf;

    #[test]
    fn test_valid_preprocessors() {
        let content = r#"# Chapter

Here's a mermaid diagram:

```mermaid
graph TD
    A --> B
```

Some math: $x = y^2$

Display math:
$$
E = mc^2
$$

```admonish note
This is a note.
```
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_empty_mermaid_block() {
        let content = r#"```mermaid
```"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Empty mermaid block"));
    }

    #[test]
    fn test_invalid_mermaid_syntax() {
        let content = "```mermaid with extra text";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should start with '```mermaid' on its own line")
        );
    }

    #[test]
    fn test_unclosed_inline_math() {
        let content = "This is $unclosed math";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Unclosed inline math"));
    }

    #[test]
    fn test_empty_math_blocks() {
        let content = r#"Empty inline: $ $
Empty display: $$$$"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("Empty inline math"));
        assert!(violations[1].message.contains("Empty display math"));
    }

    #[test]
    fn test_invalid_admonish_type() {
        let content = "```admonish invalid";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Invalid admonish type 'invalid'")
        );
    }

    #[test]
    fn test_missing_admonish_type() {
        let content = "```admonish";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Admonish block missing type")
        );
    }

    #[test]
    fn test_shell_prompts_not_math() {
        let content = r#"# Shell Commands

$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ cargo build --release
$ rustc main.rs
$ echo $PATH

Some actual math: $x = y^2$
But this is unclosed math: $x = y"#;

        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        // Should only flag the last line as unclosed math, not the shell commands
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Unclosed inline math"));
        assert_eq!(violations[0].line, 9); // The "But this is unclosed math" line
    }

    #[test]
    fn test_powershell_prompts() {
        let content = r#"> rustc main.rs
> echo %PATH%
> dir /B"#;

        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        // PowerShell prompts should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_dollar_signs_in_code_blocks() {
        let content = r#"# Shell Examples

```bash
echo "Database status: $status"
if [ "$status" = "active" ]; then
    echo "System is $status"
    export PATH=$PATH:/usr/local/bin
fi

# Variables with dollar signs
name=$1
echo "Hello $name"
result=$(echo $value | grep pattern)
```

Regular text with math: $x = y^2$

~~~sh
# Another code block style
echo $HOME
echo $USER
~~~

More text after code block"#;

        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        // Should not flag any violations for dollar signs in code blocks
        assert_eq!(
            violations.len(),
            0,
            "Dollar signs in code blocks should be ignored"
        );
    }

    #[test]
    fn test_mixed_code_and_math() {
        let content = r#"# Mixed Content

Some inline math: $x = y^2$

```bash
# Shell code with dollars
echo $PATH
export VAR=$VALUE
```

Unclosed math: $x = y

```python
# Python code (no dollars but testing code block boundaries)
print("Hello")
```

Valid display math:
$$
E = mc^2
$$"#;

        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        // Should only flag the unclosed math outside code blocks
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Unclosed inline math"));
        assert_eq!(violations[0].line, 11); // Line with "Unclosed math: $x = y"
    }
}
