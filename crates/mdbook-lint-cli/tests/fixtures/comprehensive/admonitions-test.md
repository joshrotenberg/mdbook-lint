# Admonitions and Callouts Test File

This file tests various admonition/callout syntaxes across different markdown processors.

---

## GitHub-Style Alerts (GFM)

> [!NOTE]
> Useful information that users should know, even when skimming content.

> [!TIP]
> Helpful advice for doing things better or more easily.

> [!IMPORTANT]
> Key information users need to know to achieve their goal.

> [!WARNING]
> Urgent info that needs immediate user attention to avoid problems.

> [!CAUTION]
> Advises about risks or negative outcomes of certain actions.

### With Content

> [!NOTE]
> This is a note with multiple paragraphs.
>
> Second paragraph here with **bold** and *italic* text.
>
> - List item 1
> - List item 2
>
> ```python
> print("Code in alert!")
> ```

### Nested

> [!WARNING]
> Outer warning
>
> > [!NOTE]
> > Nested note inside warning

---

## Obsidian-Style Callouts

> [!note]
> Standard note callout

> [!abstract]
> Abstract or summary callout

> [!summary]
> Summary callout (alias for abstract)

> [!tldr]
> TL;DR callout (alias for abstract)

> [!info]
> Information callout

> [!todo]
> Todo item callout

> [!tip]
> Tip or hint callout

> [!hint]
> Hint callout (alias for tip)

> [!success]
> Success callout

> [!check]
> Check callout (alias for success)

> [!done]
> Done callout (alias for success)

> [!question]
> Question callout

> [!help]
> Help callout (alias for question)

> [!faq]
> FAQ callout (alias for question)

> [!warning]
> Warning callout

> [!caution]
> Caution callout (alias for warning)

> [!attention]
> Attention callout (alias for warning)

> [!failure]
> Failure callout

> [!fail]
> Fail callout (alias for failure)

> [!missing]
> Missing callout (alias for failure)

> [!danger]
> Danger callout

> [!error]
> Error callout (alias for danger)

> [!bug]
> Bug callout

> [!example]
> Example callout

> [!quote]
> Quote callout

> [!cite]
> Citation callout (alias for quote)

### Custom Titles

> [!note] Custom Title Here
> Callout with a custom title instead of the default.

> [!tip] Pro Tip
> This tip has a custom title.

### Foldable Callouts

> [!faq]- Click to Expand (Collapsed by Default)
> This content is hidden until clicked.

> [!info]+ Expanded by Default but Foldable
> This content is visible but can be collapsed.

### Empty Title

> [!note]-
> Callout with empty title (just icon)

---

## Docusaurus-Style Admonitions

:::note
This is a note admonition.
:::

:::tip
This is a tip admonition.
:::

:::info
This is an info admonition.
:::

:::caution
This is a caution admonition.
:::

:::warning
This is a warning admonition.
:::

:::danger
This is a danger admonition.
:::

### With Titles

:::note[Note Title]
Note with a custom title.
:::

:::tip[Pro Tip]
Tip with a custom title.
:::

:::danger[Critical Issue]
Danger with a custom title.
:::

### Nested Content

:::note

This note has complex content:

1. Numbered list
2. With multiple items
3. And **formatting**

```javascript
const example = "code block in admonition";
```

> Even nested blockquotes!

:::

### Nested Admonitions

:::warning

This is an outer warning.

:::tip
This is a nested tip inside the warning.
:::

Back to the warning.

:::

---

## MkDocs Material Admonitions

!!! note
    This is a note.

!!! abstract
    This is an abstract.

!!! info
    This is info.

!!! tip
    This is a tip.

!!! success
    This is a success message.

!!! question
    This is a question.

!!! warning
    This is a warning.

!!! failure
    This is a failure message.

!!! danger
    This is danger.

!!! bug
    This is a bug report.

!!! example
    This is an example.

!!! quote
    This is a quote.

### With Titles

!!! note "Custom Note Title"
    Note with a custom title.

!!! tip "Pro Tip"
    Tip with a custom title.

!!! danger "Critical Security Issue"
    Danger with a custom title.

### Without Title

!!! note ""
    Note without any title.

### Collapsible (Details)

??? note
    This is collapsed by default.

???+ note
    This is expanded by default but collapsible.

??? note "Collapsed with Title"
    Collapsed admonition with custom title.

### Inline Admonitions

!!! info inline
    This is an inline admonition on the left.

!!! info inline end
    This is an inline admonition on the right.

### Nested Content

!!! example

    ```python
    def hello():
        print("Code in admonition")
    ```

    | Header 1 | Header 2 |
    |----------|----------|
    | Cell 1   | Cell 2   |

    1. List item
    2. Another item

---

## mdbook-admonish Syntax

```admonish note
This is a note.
```

```admonish info
This is info.
```

```admonish tip
This is a tip.
```

```admonish warning
This is a warning.
```

```admonish danger
This is danger.
```

```admonish bug
This is a bug.
```

```admonish example
This is an example.
```

```admonish quote
This is a quote.
```

```admonish success
This is a success message.
```

```admonish question
This is a question.
```

```admonish failure
This is a failure.
```

```admonish abstract
This is an abstract.
```

### With Titles

```admonish note title="My Custom Title"
Note with a custom title.
```

```admonish warning title="Important Warning"
Warning with custom title and content.
```

### Collapsible

```admonish tip collapsible=true
This tip is collapsible.
```

```admonish note collapsible=true title="Click to Expand"
Collapsed note with custom title.
```

### Complex Content

```admonish example
Here's some code:

    fn main() {
        println!("Hello!");
    }

And a list:

- Item 1
- Item 2
```

---

## reStructuredText Style (for comparison)

.. note::
   This is how notes look in RST/Sphinx.

.. warning::
   This is a warning in RST style.

.. danger::
   Danger in RST style.

---

## AsciiDoc Style (for comparison)

[NOTE]
====
This is an AsciiDoc note block.
====

[TIP]
====
This is an AsciiDoc tip.
====

[IMPORTANT]
====
This is important in AsciiDoc.
====

[WARNING]
====
This is a warning in AsciiDoc.
====

[CAUTION]
====
Caution in AsciiDoc style.
====

---

## Pandoc Fenced Divs

::: {.note}
This is a note using Pandoc fenced divs.
:::

::: {.warning}
This is a warning.
:::

::: {.tip}
This is a tip.
:::

::: {.callout-note}
Callout style note.
:::

::: {.callout-warning}
Callout style warning.
:::

::: {.callout-tip}
Callout style tip.
:::

### With Attributes

::: {.note .custom-class #custom-id}
Note with class and ID.
:::

::: {.callout-note appearance="simple"}
Simple appearance callout.
:::

::: {.callout-note collapse="true"}
Collapsible callout.
:::

---

## Hugo Shortcodes (Template Syntax)

{{< hint info >}}
This is a Hugo hint shortcode.
{{< /hint >}}

{{< hint warning >}}
Warning hint.
{{< /hint >}}

{{< hint danger >}}
Danger hint.
{{< /hint >}}

{{< notice note >}}
Notice shortcode.
{{< /notice >}}

{{< notice warning >}}
Warning notice.
{{< /notice >}}

{{< alert >}}
Default alert.
{{< /alert >}}

{{< alert "warning" >}}
Warning alert.
{{< /alert >}}

---

## VuePress/VitePress Containers

::: tip
This is a tip.
:::

::: warning
This is a warning.
:::

::: danger
This is dangerous.
:::

::: details
This is a details block.
:::

::: info
This is an info block.
:::

### With Titles

::: tip CUSTOM TITLE
A tip with custom title.
:::

::: danger STOP
Don't do this!
:::

::: details Click me to view the code
```js
console.log('Hello, VitePress!')
```
:::

---

## Docsify Alerts

> [!NOTE]
> An alert of type 'note'

> [!TIP]
> An alert of type 'tip'

> [!WARNING]
> An alert of type 'warning'

> [!ATTENTION]
> An alert of type 'attention'

---

## GitBook Hints

{% hint style="info" %}
Info hint in GitBook.
{% endhint %}

{% hint style="success" %}
Success hint.
{% endhint %}

{% hint style="warning" %}
Warning hint.
{% endhint %}

{% hint style="danger" %}
Danger hint.
{% endhint %}

---

## Edge Cases

### Empty Admonitions

> [!NOTE]

::: note
:::

!!! note

### Very Long Content

> [!NOTE]
> This is a very long note that contains a lot of text to test how the admonition handles wrapping and overflow. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.

### Special Characters in Title

> [!note] Title with "quotes" and <angles> & ampersand

!!! note "Title with \"escaped quotes\" and special chars: <>&"
    Content here.

### Code-Heavy Admonition

> [!TIP]
> Here's how to do it:
>
> ```bash
> npm install package
> npm run build
> npm test
> ```
>
> Then check the output:
>
> ```json
> {
>   "status": "success",
>   "code": 0
> }
> ```

### Image in Admonition

> [!NOTE]
> Here's an important diagram:
>
> ![Diagram](./images/diagram.png)
>
> Make sure to follow these steps.

### Table in Admonition

> [!INFO]
> Supported formats:
>
> | Format | Extension | Supported |
> |--------|-----------|-----------|
> | JSON   | .json     | ✅        |
> | YAML   | .yaml     | ✅        |
> | TOML   | .toml     | ✅        |
> | XML    | .xml      | ❌        |

### Math in Admonition

> [!NOTE]
> The quadratic formula is:
>
> $$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$

### Multiple Sequential Admonitions

> [!NOTE]
> First note.

> [!WARNING]
> Followed by warning.

> [!TIP]
> And then a tip.

> [!DANGER]
> Finally, danger.

---

## End of Admonitions Test File
