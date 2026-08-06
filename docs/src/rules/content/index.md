# Content Rules

Content rules look at what a page says rather than how it is formatted. They
catch documentation that is unfinished, inconsistent, or hard to follow: text
left over from drafting, headings that disagree with each other about
capitalization, links whose text says nothing, and terminology that changes
from one page to the next.

None of them are enabled by default. They express editorial preferences rather
than correctness, so a book adopts the ones it agrees with.

## Rules

| Rule | Checks |
|------|--------|
| [CONTENT001](./content001.md) | TODO, FIXME and similar markers left in prose |
| [CONTENT002](./content002.md) | Placeholder text such as lorem ipsum |
| [CONTENT003](./content003.md) | Chapters too short to be worth a page |
| [CONTENT004](./content004.md) | Heading capitalization that varies within a file |
| [CONTENT005](./content005.md) | A heading followed straight by a subheading, with nothing in between |
| [CONTENT006](./content006.md) | Internal links pointing at files that are not there |
| [CONTENT007](./content007.md) | The same idea named differently in different places |
| [CONTENT009](./content009.md) | Headings nested deeper than a reader will follow |
| [CONTENT010](./content010.md) | Link text that does not say where it goes |
| [CONTENT011](./content011.md) | Future tense describing what the software already does |

## Enabling them

```toml
[rules]
enabled = ["CONTENT001", "CONTENT010"]
```

See [Configuration](../../configuration.md) for the full syntax.
