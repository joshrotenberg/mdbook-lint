use crate::{Document, error::Result, violation::Violation};
use comrak::{Arena, nodes::AstNode};

/// Rule stability levels
#[derive(Debug, Clone, PartialEq)]
pub enum RuleStability {
    /// Rule is stable and recommended for production use
    Stable,
    /// Rule is experimental and may change
    Experimental,
    /// Rule is deprecated and may be removed in future versions
    Deprecated,
    /// Rule number reserved but never implemented
    Reserved,
}

/// Rule categories for grouping and filtering
#[derive(Debug, Clone, PartialEq)]
pub enum RuleCategory {
    /// Document structure and heading organization
    Structure,
    /// Whitespace, line length, and formatting consistency
    Formatting,
    /// Links, images, and content validation
    Content,
    /// Link-specific validation
    Links,
    /// Accessibility and usability rules
    Accessibility,
    /// mdBook-specific functionality and conventions
    MdBook,
}

/// Metadata about a rule's status, category, and properties
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// Whether the rule is deprecated
    pub deprecated: bool,
    /// Reason for deprecation (if applicable)
    pub deprecated_reason: Option<&'static str>,
    /// Suggested replacement rule (if applicable)
    pub replacement: Option<&'static str>,
    /// Rule category for grouping
    pub category: RuleCategory,
    /// Version when rule was introduced
    pub introduced_in: Option<&'static str>,
    /// Stability level of the rule
    pub stability: RuleStability,
    /// Rules that this rule overrides (for context-specific rules)
    pub overrides: Option<&'static str>,
}

impl RuleMetadata {
    /// Create metadata for a stable, active rule
    pub fn stable(category: RuleCategory) -> Self {
        Self {
            deprecated: false,
            deprecated_reason: None,
            replacement: None,
            category,
            introduced_in: None,
            stability: RuleStability::Stable,
            overrides: None,
        }
    }

    /// Create metadata for a deprecated rule
    pub fn deprecated(
        category: RuleCategory,
        reason: &'static str,
        replacement: Option<&'static str>,
    ) -> Self {
        Self {
            deprecated: true,
            deprecated_reason: Some(reason),
            replacement,
            category,
            introduced_in: None,
            stability: RuleStability::Deprecated,
            overrides: None,
        }
    }

    /// Create metadata for an experimental rule
    pub fn experimental(category: RuleCategory) -> Self {
        Self {
            deprecated: false,
            deprecated_reason: None,
            replacement: None,
            category,
            introduced_in: None,
            stability: RuleStability::Experimental,
            overrides: None,
        }
    }

    /// Create metadata for a reserved rule number (never implemented)
    pub fn reserved(reason: &'static str) -> Self {
        Self {
            deprecated: false,
            deprecated_reason: Some(reason),
            replacement: None,
            category: RuleCategory::Structure,
            introduced_in: None,
            stability: RuleStability::Reserved,
            overrides: None,
        }
    }

    /// Set the version when this rule was introduced
    pub fn introduced_in(mut self, version: &'static str) -> Self {
        self.introduced_in = Some(version);
        self
    }

    /// Set which rule this rule overrides
    pub fn overrides(mut self, rule_id: &'static str) -> Self {
        self.overrides = Some(rule_id);
        self
    }
}

/// Trait that all linting rules must implement
pub trait Rule: Send + Sync {
    /// Unique identifier for the rule (e.g., "MD001")
    fn id(&self) -> &'static str;

    /// Human-readable name for the rule (e.g., "heading-increment")
    fn name(&self) -> &'static str;

    /// Description of what the rule checks
    fn description(&self) -> &'static str;

    /// Metadata about this rule's status and properties
    fn metadata(&self) -> RuleMetadata;

    /// Check a document for violations of this rule with optional pre-parsed AST
    fn check_with_ast<'a>(
        &self,
        document: &Document,
        ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>>;

    /// Check a document for violations of this rule (backward compatibility)
    fn check(&self, document: &Document) -> Result<Vec<Violation>> {
        self.check_with_ast(document, None)
    }

    /// Whether this rule can automatically fix violations
    fn can_fix(&self) -> bool {
        false
    }

    /// Attempt to fix a violation (if supported)
    fn fix(&self, _content: &str, _violation: &Violation) -> Option<String> {
        None
    }

    /// Create a violation for this rule
    fn create_violation(
        &self,
        message: String,
        line: usize,
        column: usize,
        severity: crate::violation::Severity,
    ) -> Violation {
        Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            message,
            line,
            column,
            severity,
            fix: None,
        }
    }

    /// Create a violation with a fix for this rule
    fn create_violation_with_fix(
        &self,
        message: String,
        line: usize,
        column: usize,
        severity: crate::violation::Severity,
        fix: crate::violation::Fix,
    ) -> Violation {
        Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            message,
            line,
            column,
            severity,
            fix: Some(fix),
        }
    }
}

/// Helper trait for AST-based rules
///
/// # When to Use AstRule vs Rule
///
/// **Use `AstRule` when your rule needs to:**
/// - Analyze document structure (headings, lists, links, code blocks)
/// - Navigate parent-child relationships in the markdown tree
/// - Access precise position information from comrak's sourcepos
/// - Understand markdown semantics beyond simple text patterns
///
/// **Use `Rule` directly when your rule:**
/// - Only needs line-by-line text analysis
/// - Checks simple text patterns (trailing spaces, line length)
/// - Doesn't need to understand markdown structure
///
/// # Implementation Examples
///
/// **AstRule Examples:**
/// - `MD001` (heading-increment): Needs to traverse heading hierarchy
/// - `MDBOOK002` (link-validation): Needs to find and validate link nodes
/// - `MD031` (blanks-around-fences): Needs to identify fenced code blocks
///
/// **Rule Examples:**
/// - `MD013` (line-length): Simple line-by-line character counting
/// - `MD009` (no-trailing-spaces): Pattern matching on line endings
///
/// # Basic Implementation Pattern
///
/// ```rust
/// use mdbook_lint_core::rule::{AstRule, RuleMetadata, RuleCategory};
/// use mdbook_lint_core::{Document, Violation, Result};
/// use comrak::nodes::{AstNode, NodeValue};
///
/// pub struct MyRule;
///
/// impl AstRule for MyRule {
///     fn id(&self) -> &'static str { "MY001" }
///     fn name(&self) -> &'static str { "my-rule" }
///     fn description(&self) -> &'static str { "Description of what this rule checks" }
///
///     fn metadata(&self) -> RuleMetadata {
///         RuleMetadata::stable(RuleCategory::Structure)
///     }
///
///     fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
///         let mut violations = Vec::new();
///
///         // Find nodes of interest
///         for node in ast.descendants() {
///             if let NodeValue::Heading(heading) = &node.data.borrow().value {
///                 // Get position information
///                 if let Some((line, column)) = document.node_position(node) {
///                     // Check some condition
///                     if heading.level > 3 {
///                         violations.push(self.create_violation(
///                             "Heading too deep".to_string(),
///                             line,
///                             column,
///                             mdbook_lint_core::violation::Severity::Warning,
///                         ));
///                     }
///                 }
///             }
///         }
///
///         Ok(violations)
///     }
/// }
/// ```
///
/// # Key Methods Available
///
/// **From Document:**
/// - `document.node_position(node)` - Get (line, column) for any AST node
/// - `document.node_text(node)` - Extract text content from a node
/// - `document.headings(ast)` - Get all heading nodes
/// - `document.code_blocks(ast)` - Get all code block nodes
///
/// **From AstNode:**
/// - `node.descendants()` - Iterate all child nodes recursively
/// - `node.children()` - Get direct children only
/// - `node.parent()` - Get parent node (if any)
/// - `node.data.borrow().value` - Access the NodeValue enum
///
/// **Creating Violations:**
/// - `self.create_violation(message, line, column, severity)` - Standard violation creation
/// - `self.create_violation_with_fix(message, line, column, severity, fix)` - Violation with fix
pub trait AstRule: Send + Sync {
    /// Unique identifier for the rule (e.g., "MD001")
    fn id(&self) -> &'static str;

    /// Human-readable name for the rule (e.g., "heading-increment")
    fn name(&self) -> &'static str;

    /// Description of what the rule checks
    fn description(&self) -> &'static str;

    /// Metadata about this rule's status and properties
    fn metadata(&self) -> RuleMetadata;

    /// Check a document using its AST
    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>>;

    /// Whether this rule can automatically fix violations
    fn can_fix(&self) -> bool {
        false
    }

    /// Attempt to fix a violation (if supported)
    fn fix(&self, _content: &str, _violation: &Violation) -> Option<String> {
        None
    }

    /// Create a violation for this rule
    fn create_violation(
        &self,
        message: String,
        line: usize,
        column: usize,
        severity: crate::violation::Severity,
    ) -> Violation {
        Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            message,
            line,
            column,
            severity,
            fix: None,
        }
    }

    /// Create a violation with a fix for this rule
    fn create_violation_with_fix(
        &self,
        message: String,
        line: usize,
        column: usize,
        severity: crate::violation::Severity,
        fix: crate::violation::Fix,
    ) -> Violation {
        Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            message,
            line,
            column,
            severity,
            fix: Some(fix),
        }
    }
}

// Blanket implementation so AstRule types automatically implement Rule
impl<T: AstRule> Rule for T {
    fn id(&self) -> &'static str {
        T::id(self)
    }

    fn name(&self) -> &'static str {
        T::name(self)
    }

    fn description(&self) -> &'static str {
        T::description(self)
    }

    fn metadata(&self) -> RuleMetadata {
        T::metadata(self)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        if let Some(ast) = ast {
            self.check_ast(document, ast)
        } else {
            let arena = Arena::new();
            let ast = document.parse_ast(&arena);
            self.check_ast(document, ast)
        }
    }

    fn check(&self, document: &Document) -> Result<Vec<Violation>> {
        self.check_with_ast(document, None)
    }

    fn can_fix(&self) -> bool {
        T::can_fix(self)
    }

    fn fix(&self, content: &str, violation: &Violation) -> Option<String> {
        T::fix(self, content, violation)
    }

    fn create_violation(
        &self,
        message: String,
        line: usize,
        column: usize,
        severity: crate::violation::Severity,
    ) -> Violation {
        T::create_violation(self, message, line, column, severity)
    }
}
