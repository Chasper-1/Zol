# zml Markup Syntax

Zol uses **zml** (Zol Markup Language) — a lightweight semantic markup with strict pairing rules. Files use the `.zml` extension.

## Inline Formatting

| Marker | Style | Example | Renders as |
|--------|-------|---------|------------|
| `**text**` | Bold | `**important**` | **important** |
| `//text//` | Italic | `//emphasis//` | *emphasis* |
| `__text__` | Underline | `__underlined__` | <u>underlined</u> |
| `~~text~~` | Strikethrough | `~~deleted~~` | ~~deleted~~ |
| `''text''` | Superscript | `''2''` | ² |
| `,,text,,` | Subscript | `,,2,,` | ₂ |
| `` `text` `` | Code | `` `fn()` `` | `fn()` |
| `==text==` | Highlight | `==important==` | (highlighted) |
| `!!text!!` | Spoiler (inline) | `!!secret!!` | (hidden) |
| `!!!text!!!` | Spoiler (block) | `!!!spoiler!!!` | (hidden block) |
| `++text++` | Insertion | `++new++` | (green text) |
| `--text--` | Deletion | `--old--` | (red + strikethrough) |
| `/*text*\` | Comment | `/*not visible*\` | (hidden) |
| `$text$` | Inline formula | `$x+y$` | (monospace green) |
| `$$text$$` | Display formula | `$$x^2$$` | (centered monospace) |

## Parser Rules

- **Strict pairing** — unclosed markers are plain text
- **No whitespace** adjacent to markers
- **Non-empty content** required between markers
- **Escape** with backslash `\`
- **Longer markers** have priority (`$$` before `$`, `!!!` before `!!`)
- **Nesting** allowed, LIFO stack
- **Multiline** markers: `/* ... *\`, `$$ ... $$`, `!!! ... !!!`

## Internal Pipeline

```
Raw text → token::tokenize() → parser::parse() → segmenter::to_document_cache()
```

15 style flags, 30+ unit tests.
