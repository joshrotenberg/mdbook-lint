# MDBOOK011 - Invalid Template Syntax

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates `{{#template}}` directives used with the mdbook-template preprocessor. It ensures correct syntax for template inclusion and variable substitution.

## Why This Rule Exists

Valid template syntax is important because:
- Prevents build failures from malformed templates
- Ensures consistent content across chapters
- Enables proper variable substitution
- Maintains template reusability
- Helps identify template errors early

## Examples

### ❌ Incorrect (violates rule)

```markdown
<!-- Invalid directive syntax -->
{{template file.md}}
{{#templates file.md}}

<!-- Missing template file -->
{{#template ./missing-template.md}}

<!-- Invalid variable syntax -->
{{#template ./template.md
    name=value with spaces
    key = value
}}

<!-- Unclosed directive -->
{{#template ./template.md
    var1=value1
```

### ✅ Correct

```markdown
<!-- Basic template inclusion -->
{{#template ./templates/header.md}}

<!-- Template with variables -->
{{#template ./templates/api-doc.md
    method=GET
    endpoint=/api/users
    description=Retrieves all users
}}

<!-- Multi-line variables -->
{{#template ./templates/example.md
    title=Introduction
    content=This is the content
    footer=Copyright 2024
}}
```

## Template File Syntax

### Template Definition

```markdown
<!-- templates/api-doc.md -->
## {{method}} {{endpoint}}

{{description}}

### Request
```http
{{method}} {{endpoint}} HTTP/1.1
Host: api.example.com
```

### Response
```json
{
    "status": "success",
    "data": {{response_data}}
}
```
```

### Using the Template

```markdown
{{#template ./templates/api-doc.md
    method=POST
    endpoint=/api/users
    description=Creates a new user
    response_data={"id": 123, "name": "John"}
}}
```

## Configuration

```toml
# book.toml
[preprocessor.template]

# .mdbook-lint.toml
[MDBOOK011]
template_dir = "./templates"  # Default template directory
check_variables = true         # Validate variable substitution
allow_missing_vars = false     # Allow undefined variables
```

## Common Issues and Solutions

### Issue: Spaces in Variable Values
```markdown
<!-- Wrong: Unquoted spaces -->
{{#template ./template.md
    title=My Great Title
}}

<!-- Correct: Use quotes or underscores -->
{{#template ./template.md
    title="My Great Title"
}}

<!-- Or use underscores -->
{{#template ./template.md
    title=My_Great_Title
}}
```

### Issue: Variable Not Replaced
```markdown
<!-- Template -->
Hello {{name}}!

<!-- Wrong variable name -->
{{#template ./greeting.md
    username=Alice  <!-- Should be 'name' -->
}}

<!-- Correct -->
{{#template ./greeting.md
    name=Alice
}}
```

### Issue: Nested Templates
```markdown
<!-- templates/outer.md -->
# {{title}}

{{#template ./inner.md
    content={{inner_content}}
}}

<!-- Using nested templates -->
{{#template ./templates/outer.md
    title=Documentation
    inner_content=Details here
}}
```

## Best Practices

1. **Organize templates**: Keep in dedicated directory
2. **Name variables clearly**: Use descriptive names
3. **Document variables**: List required variables in template
4. **Provide defaults**: Handle missing variables gracefully
5. **Test substitution**: Verify all variables are replaced

### Template Organization

```
book/
├── src/
│   ├── chapter1.md
│   └── chapter2.md
├── templates/
│   ├── api/
│   │   ├── request.md
│   │   └── response.md
│   ├── components/
│   │   ├── header.md
│   │   └── footer.md
│   └── examples/
│       └── code-block.md
```

### Self-Documenting Templates

```markdown
<!-- templates/documented.md -->
<!--
Template: API Endpoint Documentation
Required variables:
  - method: HTTP method (GET, POST, etc.)
  - path: API endpoint path
  - description: What this endpoint does
Optional variables:
  - params: Query parameters
  - body: Request body example
-->

## {{method}} {{path}}

{{description}}

{{#if params}}
### Parameters
{{params}}
{{/if}}

{{#if body}}
### Request Body
```json
{{body}}
```
{{/if}}
```

## Advanced Usage

### Conditional Content

```markdown
<!-- Template with conditional sections -->
{{#if premium}}
This content is only for premium users.
{{/if}}

{{#unless beta}}
This feature is available in stable release.
{{/unless}}
```

### Loops and Lists

```markdown
<!-- Template with repeated content -->
{{#each items}}
- {{this.name}}: {{this.description}}
{{/each}}
```

### Default Values

```markdown
<!-- Template with defaults -->
# {{title|Default Title}}

Author: {{author|Anonymous}}
Date: {{date|TBD}}
```

## When to Disable

Consider disabling this rule if:
- You don't use the template preprocessor
- You use a different template system
- Your templates are generated dynamically
- You're migrating template syntax

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK011"]
```

### Disable Inline

```markdown
<!-- mdbook-lint-disable MDBOOK011 -->
{{#template ./experimental-template.md}}
<!-- mdbook-lint-enable MDBOOK011 -->
```

## Error Messages

| Error | Solution |
|-------|----------|
| "Template file not found" | Check file path and existence |
| "Invalid variable syntax" | Use key=value format |
| "Undefined variable" | Ensure all variables are provided |
| "Unclosed template directive" | Add closing }} |

## Related Rules

- [MDBOOK007](./mdbook007.html) - Include directive validation
- [MDBOOK012](./mdbook012.html) - Include line ranges

## References

- [mdbook-template](https://github.com/sgoudham/mdbook-template)
- [mdBook Preprocessors](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html)
- [Handlebars Template Syntax](https://handlebarsjs.com/)