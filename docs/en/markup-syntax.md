# md+ Markup Syntax

Zol uses the **md+** markup language — a lightweight semantic markup with strict pairing rules.

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

### 1. Strict Pairing

Formatting is applied **only** when both opening and closing markers are found. An unclosed marker is treated as plain text.

```
**bold**        → ✅ bold
**not bold      → ❌ **not bold (plain text)
```

### 2. Whitespace Adjacent to Markers

No whitespace allowed immediately after opening marker or before closing marker.

```
**text**         → ✅
** text**        → ❌ (space after **)
**text **        → ❌ (space before **)
```

Whitespace *inside* content is fine:
```
**bold text**    → ✅
```

Line boundaries are valid — whitespace is not required.

### 3. Non-empty Content

At least one non-whitespace character must exist between markers.

```
****            → ❌ plain text
////            → ❌ plain text
```

### 4. Escape

Backslash `\` before any character removes its special meaning. The backslash itself is not displayed.

```
\*\*text\*\*     → **text** (plain)
\\               → \
```

### 5. Marker Priority

When markers share a prefix, the longer one wins:

```
$$x$$           → display formula ($$ wins)
$x$             → inline formula
!!!text!!!      → block spoiler
!!text!!        → inline spoiler
```

Markers are checked longest-first.

### 6. Nesting

Markers can nest. The stack closes in reverse order (LIFO).

```
**bold //and italic//**         → ✅
//italic inside **bold**//       → ✅
**a **b** c**                    → ✅ (nested bold)
```

Styles combine: nested markers add their style bits to the parent.

### 7. Multiline Markers

The following markers work across multiple lines:

| Marker | Type |
|--------|------|
| `/* ... *\` | Comment |
| `$$ ... $$` | Display formula |
| `!!! ... !!!` | Block spoiler |

Regular markers (`**`, `//`, `~~`, etc.) close within the **same line**.

### 8. Comment Variant

md+ uses `/*text*\` (backslash-asterisk) instead of the traditional `*/` to avoid conflicts with inline markers that might contain `*`.

## Internal Implementation

The parser pipeline:

```
Raw text → token::tokenize() → Vec<Token>
         → parser::parse()   → MarkupDoc (AST)
         → segmenter::to_document_cache() → DocumentCache
```

- **Tokenizer**: Single pass, O(n). Produces `Token::Text`, `Token::Open(MarkupStyle)`, `Token::Close(MarkupStyle)`, `Token::Newline`.
- **Parser**: Stack-based, O(n). Builds `MarkupDoc` with nested `MarkupNode::Formatted` nodes.
- **Segmenter**: Flattens AST into `DocumentCache` — a vec of `MarkupCache` (one per line), each containing a list of `Segment` with byte offsets and style bitmask.

### Style Flags

```rust
MarkupStyle(bitmask):
  PLAIN           = 0
  BOLD            = 1 << 0
  ITALIC          = 1 << 1
  UNDERLINE       = 1 << 2
  STRIKETHROUGH   = 1 << 3
  SUPERSCRIPT     = 1 << 4
  SUBSCRIPT       = 1 << 5
  CODE            = 1 << 6
  HIGHLIGHT       = 1 << 7
  SPOILER         = 1 << 8
  SPOILER_BLOCK   = 1 << 9
  INSERTION       = 1 << 10
  DELETION        = 1 << 11
  COMMENT         = 1 << 12
  FORMULA         = 1 << 13
  DISPLAY_FORMULA = 1 << 14
```
