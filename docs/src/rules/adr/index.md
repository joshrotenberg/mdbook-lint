# ADR (Architecture Decision Record) Rules

These rules validate Architecture Decision Records (ADRs) against the Nygard format and MADR 4.0 format, ensuring consistency and completeness in your architectural documentation.

## Rules

### Structure Rules

| Rule ID | Name | Description |
|---------|------|-------------|
| [ADR001](./adr001.md) | adr-title-format | Title follows appropriate format for ADR type |
| [ADR002](./adr002.md) | adr-required-status | Status is defined (section or frontmatter) |
| [ADR003](./adr003.md) | adr-required-date | Date is defined (line or frontmatter) |
| [ADR004](./adr004.md) | adr-required-context | Context section is present |
| [ADR005](./adr005.md) | adr-required-decision | Decision section is present |
| [ADR006](./adr006.md) | adr-required-consequences | Consequences section is present (Nygard only) |

### Validation Rules

| Rule ID | Name | Description |
|---------|------|-------------|
| [ADR007](./adr007.md) | adr-valid-status | Status value is recognized |
| [ADR008](./adr008.md) | adr-date-format | Date follows ISO 8601 format |
| [ADR009](./adr009.md) | adr-filename-matches-number | Filename matches ADR number (Nygard only) |

### Collection Rules (Multi-Document)

| Rule ID | Name | Description |
|---------|------|-------------|
| [ADR010](./adr010.md) | adr-superseded-has-replacement | Superseded ADRs reference replacement |
| [ADR011](./adr011.md) | adr-sequential-numbering | ADR numbers are sequential with no gaps |
| [ADR012](./adr012.md) | adr-no-duplicate-numbers | Each ADR number is unique |
| [ADR013](./adr013.md) | adr-valid-adr-links | Links to other ADRs point to existing files |

### Content Quality Rules

| Rule ID | Name | Description |
|---------|------|-------------|
| [ADR014](./adr014.md) | adr-non-empty-sections | Required sections should have meaningful content |
| [ADR015](./adr015.md) | adr-decision-drivers-format | Decision Drivers should be a bullet list (MADR) |
| [ADR016](./adr016.md) | adr-considered-options-format | Considered Options should list at least 2 options |
| [ADR017](./adr017.md) | adr-consequences-structure | Consequences should distinguish good/bad outcomes (MADR) |

## Supported Formats

### Nygard Format

The original ADR format proposed by Michael Nygard. Key characteristics:

- Title: `# N. Title` (e.g., `# 1. Record architecture decisions`)
- Date: `Date: YYYY-MM-DD` line after the title
- Status: `## Status` section with status value
- Required sections: Context, Decision, Consequences

```markdown
# 1. Record architecture decisions

Date: 2024-01-15

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as described by Michael Nygard.

## Consequences

See Michael Nygard's article for more details.
```

### MADR 4.0 Format

Markdown Any Decision Records (MADR) version 4.0 uses YAML frontmatter for metadata and a different structure:

- YAML frontmatter with `status` and `date` fields
- Simple H1 title (no number prefix required)
- Different section names (Context and Problem Statement, Decision Outcome)

```markdown
---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice Smith
consulted:
  - Bob Jones
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database for our application.

## Decision Drivers

* Need ACID compliance
* Team familiarity with SQL

## Considered Options

* PostgreSQL
* MySQL
* MongoDB

## Decision Outcome

Chosen option: PostgreSQL, because it provides ACID compliance
and the team has extensive SQL experience.

### Consequences

* Good, because mature ecosystem
* Bad, because requires operational overhead
```

## Format Detection

The rules automatically detect the ADR format based on:

1. **YAML frontmatter present** - MADR 4.0 format
2. **No frontmatter, numbered title** - Nygard format
3. **Path contains `/adr/` or `/adrs/`** - Treated as ADR document

## Configuration

Configure ADR rules in your `.mdbook-lint.toml`:

```toml
# Enable all ADR rules (they're enabled by default)
[rules]
"ADR*" = true

# Configure valid status values
[ADR007]
valid-statuses = ["proposed", "accepted", "deprecated", "superseded", "rejected"]

# Customize minimum options for Considered Options
[ADR016]
min-options = 2
```

## Why ADR Rules Matter

Architecture Decision Records are critical for:

1. **Knowledge Transfer**: New team members understand past decisions
2. **Decision Quality**: Forces structured thinking about alternatives
3. **Accountability**: Documents who made decisions and why
4. **Reversibility**: Makes it clear when to revisit decisions
5. **Consistency**: Ensures all ADRs follow the same format

## Common Issues

### Missing Status

**Problem**: ADR doesn't indicate its current status.

```markdown
# 1. Use Rust

Date: 2024-01-15

## Context

We need a language.
```

**Solution**: Add a Status section.

```markdown
# 1. Use Rust

Date: 2024-01-15

## Status

Proposed
```

### Placeholder Content

**Problem**: Sections contain placeholder text instead of real content.

```markdown
## Context

TODO: Fill in context
```

**Solution**: Write meaningful content or mark the ADR as draft.

### Superseded Without Reference

**Problem**: ADR is superseded but doesn't link to the replacement.

```markdown
## Status

Superseded
```

**Solution**: Reference the new ADR.

```markdown
## Status

Superseded by [ADR-0005](0005-use-kubernetes.md)
```

## Integration with CI/CD

Validate ADRs in your pipeline:

```yaml
# .github/workflows/adr-check.yml
name: ADR Validation

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install mdbook-lint
        run: cargo install mdbook-lint
      - name: Validate ADRs
        run: mdbook-lint lint docs/adr/*.md --enable "ADR*"
```

## Best Practices

1. **Number ADRs sequentially**: Don't reuse numbers, even for rejected ADRs
2. **Keep ADRs immutable**: Create new ADRs to supersede old ones
3. **Link related ADRs**: Reference related decisions
4. **Include context**: Future readers need to understand the situation
5. **Document alternatives**: Show what was considered and why it was rejected
6. **Update status promptly**: Keep the status current

## References

- [ADR GitHub Organization](https://adr.github.io/)
- [Michael Nygard's original article](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [MADR 4.0.0 Specification](https://adr.github.io/madr/)
- [ADR Tools](https://github.com/npryce/adr-tools)
