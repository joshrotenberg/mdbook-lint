//! YAML frontmatter parsing for MADR 4.0 format
//!
//! MADR 4.0 ADRs use YAML frontmatter to store metadata like status and date.

use serde::Deserialize;

/// MADR 4.0 frontmatter fields
///
/// Example:
/// ```yaml
/// ---
/// status: accepted
/// date: 2024-01-15
/// decision-makers:
///   - Alice Smith
///   - Bob Jones
/// consulted:
///   - Security Team
/// informed:
///   - Engineering Team
/// ---
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AdrFrontmatter {
    /// ADR status (e.g., "proposed", "accepted", "deprecated", "superseded")
    pub status: Option<String>,

    /// Date of the decision (typically YYYY-MM-DD format)
    pub date: Option<String>,

    /// List of decision makers
    #[serde(rename = "decision-makers", alias = "deciders")]
    pub decision_makers: Option<Vec<String>>,

    /// List of people/teams consulted
    pub consulted: Option<Vec<String>>,

    /// List of people/teams to be informed
    pub informed: Option<Vec<String>>,
}

/// Result of parsing frontmatter from a document
#[derive(Debug, Clone)]
pub struct FrontmatterResult {
    /// The parsed frontmatter (if valid YAML)
    pub frontmatter: Option<AdrFrontmatter>,

    /// The line number where the opening `---` is found (1-indexed)
    pub start_line: usize,

    /// The line number where the closing `---` is found (1-indexed)
    pub end_line: usize,

    /// Raw YAML content between the delimiters
    pub raw_yaml: String,

    /// Any parsing error message
    pub error: Option<String>,
}

/// Parse YAML frontmatter from document content
///
/// Returns `None` if no frontmatter is present (document doesn't start with `---`)
pub fn parse_frontmatter(content: &str) -> Option<FrontmatterResult> {
    let trimmed = content.trim_start();

    // Must start with ---
    if !trimmed.starts_with("---") {
        return None;
    }

    // Calculate the offset for line numbering (accounting for trimmed whitespace)
    let leading_lines = content
        .chars()
        .take(content.len() - trimmed.len())
        .filter(|&c| c == '\n')
        .count();

    let lines: Vec<&str> = trimmed.lines().collect();

    // Find the closing ---
    let mut end_idx = None;
    for (idx, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = Some(idx);
            break;
        }
    }

    let end_idx = match end_idx {
        Some(idx) => idx,
        None => {
            // No closing delimiter found
            return Some(FrontmatterResult {
                frontmatter: None,
                start_line: leading_lines + 1,
                end_line: leading_lines + 1,
                raw_yaml: String::new(),
                error: Some("No closing '---' delimiter found for frontmatter".to_string()),
            });
        }
    };

    // Extract the YAML content
    let yaml_content: String = lines[1..end_idx].join("\n");

    // Try to parse the YAML
    let (frontmatter, error) = match serde_yaml::from_str::<AdrFrontmatter>(&yaml_content) {
        Ok(fm) => (Some(fm), None),
        Err(e) => (
            None,
            Some(format!("Failed to parse YAML frontmatter: {}", e)),
        ),
    };

    Some(FrontmatterResult {
        frontmatter,
        start_line: leading_lines + 1,
        end_line: leading_lines + end_idx + 1,
        raw_yaml: yaml_content,
        error,
    })
}

/// Extract the body content (everything after the frontmatter)
pub fn extract_body(content: &str) -> &str {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return content;
    }

    // Find the closing ---
    let lines: Vec<&str> = trimmed.lines().collect();
    for (idx, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            // Return everything after the closing delimiter
            let body_start = lines[..=idx].join("\n").len() + 1; // +1 for newline
            if body_start < trimmed.len() {
                return &trimmed[body_start..];
            } else {
                return "";
            }
        }
    }

    // No closing delimiter, return entire content
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_frontmatter() {
        let content = r#"---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice Smith
---

# Title
"#;

        let result = parse_frontmatter(content).unwrap();
        assert!(result.frontmatter.is_some());
        assert!(result.error.is_none());

        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.status, Some("accepted".to_string()));
        assert_eq!(fm.date, Some("2024-01-15".to_string()));
        assert_eq!(fm.decision_makers, Some(vec!["Alice Smith".to_string()]));

        assert_eq!(result.start_line, 1);
        assert_eq!(result.end_line, 6);
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Title\n\nSome content";
        assert!(parse_frontmatter(content).is_none());
    }

    #[test]
    fn test_parse_frontmatter_unclosed() {
        let content = r#"---
status: accepted
date: 2024-01-15

# Title
"#;

        let result = parse_frontmatter(content).unwrap();
        assert!(result.frontmatter.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No closing"));
    }

    #[test]
    fn test_parse_frontmatter_with_leading_whitespace() {
        let content = "\n\n---\nstatus: accepted\n---\n\n# Title";

        let result = parse_frontmatter(content).unwrap();
        assert!(result.frontmatter.is_some());
        assert_eq!(result.start_line, 3);
        assert_eq!(result.end_line, 5);
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = "---\n---\n\n# Title";

        let result = parse_frontmatter(content).unwrap();
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert!(fm.status.is_none());
        assert!(fm.date.is_none());
    }

    #[test]
    fn test_extract_body() {
        let content = r#"---
status: accepted
---

# Title

Body content here.
"#;

        let body = extract_body(content);
        assert!(body.contains("# Title"));
        assert!(body.contains("Body content here."));
        assert!(!body.contains("status:"));
    }

    #[test]
    fn test_extract_body_no_frontmatter() {
        let content = "# Title\n\nBody content";
        assert_eq!(extract_body(content), content);
    }

    #[test]
    fn test_deciders_alias() {
        let content = r#"---
status: accepted
deciders:
  - Alice Smith
---
"#;

        let result = parse_frontmatter(content).unwrap();
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.decision_makers, Some(vec!["Alice Smith".to_string()]));
    }

    #[test]
    fn test_all_optional_fields() {
        let content = r#"---
status: proposed
date: 2024-01-20
decision-makers:
  - Alice
  - Bob
consulted:
  - Security Team
informed:
  - Engineering
---
"#;

        let result = parse_frontmatter(content).unwrap();
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.status, Some("proposed".to_string()));
        assert_eq!(fm.date, Some("2024-01-20".to_string()));
        assert_eq!(
            fm.decision_makers,
            Some(vec!["Alice".to_string(), "Bob".to_string()])
        );
        assert_eq!(fm.consulted, Some(vec!["Security Team".to_string()]));
        assert_eq!(fm.informed, Some(vec!["Engineering".to_string()]));
    }
}
