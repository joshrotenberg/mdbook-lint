//! MD014: Dollar signs used before commands without showing output
//!
//! This rule checks that shell commands in code blocks don't include dollar signs
//! as part of the command, which makes them harder to copy and paste.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check that shell commands don't include dollar signs
pub struct MD014;

impl AstRule for MD014 {
    fn id(&self) -> &'static str {
        "MD014"
    }

    fn name(&self) -> &'static str {
        "no-dollar-signs"
    }

    fn description(&self) -> &'static str {
        "Dollar signs used before commands without showing output"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Find all code block nodes
        for node in ast.descendants() {
            if let NodeValue::CodeBlock(code_block) = &node.data.borrow().value {
                let info = code_block.info.trim().to_lowercase();

                // Check if this is a shell-related code block
                if is_shell_language(&info) {
                    let content = &code_block.literal;
                    let lines: Vec<&str> = content.lines().collect();

                    for (line_idx, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();

                        // Skip empty lines and comments
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }

                        // Check if line starts with $ (potentially with whitespace)
                        if trimmed.starts_with('$') {
                            // Make sure it's not just a variable or other valid use
                            if is_command_prompt_dollar(trimmed)
                                && let Some((base_line, _)) = document.node_position(node)
                            {
                                let actual_line = base_line + line_idx + 1; // +1 because code block content starts on next line
                                violations.push(self.create_violation(
                                    format!("Shell command should not include dollar sign prompt: '{trimmed}'"),
                                    actual_line,
                                    1,
                                    Severity::Warning,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

/// Check if the language info indicates a shell-related code block
fn is_shell_language(info: &str) -> bool {
    let shell_languages = [
        "sh",
        "bash",
        "shell",
        "zsh",
        "fish",
        "csh",
        "tcsh",
        "ksh",
        "console",
        "terminal",
        "cmd",
        "powershell",
        "ps1",
    ];

    // Check if the info string starts with any shell language
    // (handles cases like "bash,no_run" or "sh copy")
    for lang in &shell_languages {
        if info == *lang
            || info.starts_with(&format!("{lang},"))
            || info.starts_with(&format!("{lang} "))
        {
            return true;
        }
    }

    false
}

/// Check if a dollar sign is being used as a command prompt
fn is_command_prompt_dollar(line: &str) -> bool {
    let trimmed = line.trim();

    // Must start with $
    if !trimmed.starts_with('$') {
        return false;
    }

    // Get the part after the $
    let after_dollar = &trimmed[1..];

    // If there's a space after $, it's likely a command prompt
    if after_dollar.starts_with(' ') {
        return true;
    }

    // If it's just $ followed by nothing, it's likely a prompt
    if after_dollar.is_empty() {
        return true;
    }

    // Don't flag common shell variable patterns
    // Like $VAR, $(command), ${var}, $((math))
    if after_dollar.starts_with('(')
        || after_dollar.starts_with('{')
        || after_dollar
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
    {
        return false;
    }

    // Don't flag multiple dollar signs ($$, $$$, etc.) - these are less likely to be prompts
    if after_dollar.starts_with('$') {
        return false;
    }

    // For anything else that looks like a command (lowercase letter after $), flag it
    // This catches cases like "$echo" or "$cd"
    if let Some(first_char) = after_dollar.chars().next() {
        first_char.is_ascii_lowercase()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md014_no_violations() {
        let content = r#"# Valid Shell Commands

These shell commands should not trigger violations:

```bash
echo "Hello, world!"
ls -la
cd /home/user
```

```sh
grep "pattern" file.txt
find . -name "*.rs"
```

Variables and substitutions are fine:

```bash
echo $HOME
echo $(date)
echo ${USER}
result=$((2 + 3))
```

Non-shell code blocks are ignored:

```rust
let x = "$not_a_shell_command";
```

```python
print("$this is fine")
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md014_dollar_sign_violations() {
        let content = r#"# Shell Commands with Dollar Signs

These should trigger violations:

```bash
$ echo "Hello, world!"
$ ls -la
```

```sh
$ cd /home/user
$ grep "pattern" file.txt
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 4);
        assert!(
            violations[0]
                .message
                .contains("Shell command should not include dollar sign prompt")
        );
        assert!(violations[0].message.contains("$ echo \"Hello, world!\""));
    }

    #[test]
    fn test_md014_mixed_valid_invalid() {
        let content = r#"# Mixed Valid and Invalid

```bash
# This is a comment
echo "This is fine"
$ echo "This is not fine"
ls -la
$ cd /home
export VAR="value"
$ grep "pattern" file.txt
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_md014_different_shell_languages() {
        let content = r#"# Different Shell Languages

```console
$ echo "console command"
```

```terminal
$ ls -la
```

```zsh
$ cd /home
```

```fish
$ grep "pattern" file.txt
```

```powershell
$ Get-Process
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 5);
    }

    #[test]
    fn test_md014_variables_not_flagged() {
        let content = r#"# Variable Usage

```bash
echo $HOME
echo $USER
echo ${HOME}/bin
echo $(date)
result=$((2 + 3))
$VAR="something"
$_PRIVATE_VAR="value"
```

These should not be flagged as they are valid shell syntax.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md014_empty_lines_and_comments() {
        let content = r#"# Empty Lines and Comments

```bash
# This is a comment
$ echo "This should be flagged"

# Another comment

$ ls -la
echo "This is fine"
# Final comment
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_md014_non_shell_languages_ignored() {
        let content = r#"# Non-Shell Languages

```javascript
console.log("$ this is fine");
```

```python
print("$ also fine")
```

```rust
println!("$ still fine");
```

```markdown
$ This is in markdown, should be ignored
```

```
$ This has no language specified, should be ignored
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md014_indented_dollar_signs() {
        let content = r#"# Indented Dollar Signs

```bash
    $ echo "indented command"
  $ echo "also indented"
$ echo "not indented"
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_md014_edge_cases() {
        let content = r#"# Edge Cases

```bash
$
$
$echo_no_space
$ echo "with space"
$$
$$$multiple
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD014;
        let violations = rule.check(&document).unwrap();

        // Should flag: $, $ , $echo_no_space, $ echo "with space"
        // Should not flag: $$, $$$multiple (these are less likely to be prompts)
        assert_eq!(violations.len(), 4);
    }

    #[test]
    fn test_shell_language_detection() {
        assert!(is_shell_language("bash"));
        assert!(is_shell_language("sh"));
        assert!(is_shell_language("shell"));
        assert!(is_shell_language("console"));
        assert!(is_shell_language("bash,no_run"));
        assert!(is_shell_language("sh copy"));

        assert!(!is_shell_language("rust"));
        assert!(!is_shell_language("python"));
        assert!(!is_shell_language("javascript"));
        assert!(!is_shell_language(""));
    }

    #[test]
    fn test_command_prompt_dollar_detection() {
        assert!(is_command_prompt_dollar("$ echo hello"));
        assert!(is_command_prompt_dollar("$"));
        assert!(is_command_prompt_dollar("$ "));
        assert!(is_command_prompt_dollar("$command"));

        assert!(!is_command_prompt_dollar("$VAR"));
        assert!(!is_command_prompt_dollar("$HOME"));
        assert!(!is_command_prompt_dollar("$(command)"));
        assert!(!is_command_prompt_dollar("${var}"));
        assert!(!is_command_prompt_dollar("$((math))"));
        assert!(!is_command_prompt_dollar("$_PRIVATE"));
    }
}
