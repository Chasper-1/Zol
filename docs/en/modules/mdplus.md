# md+ Parser Module

`src/mdplus/` — standalone markup parser for the **md+** language.

```
mdplus/
├── mod.rs        — Public API: parse_document()
├── ast.rs        — AST types (MarkupDoc, MarkupNode, MarkupStyle)
├── token.rs      — Tokenizer
├── parser.rs     — Stack-based AST builder
└── segmenter.rs  — AST → DocumentCache conversion
```

## Pipeline

```
Raw text
    │
    ▼
token::tokenize(text)  →  Vec<Token>
    │
    ▼
parser::parse(&tokens)  →  MarkupDoc (AST)
    │
    ▼
segmenter::to_document_cache(&ast)  →  DocumentCache
    │
    ▼
editor::layout::compute_line_runs()  →  Vec<TextRun>
```

## Token Types

```rust
pub enum Token {
    Text(String),                   // Plain text fragment
    Open(MarkupStyle),              // Opening marker
    Close(MarkupStyle),             // Closing marker
    Newline,                        // Line break
}
```

The tokenizer is:
- **Single pass** — O(n), no backtracking
- **Escape-aware** — `\X` produces a `Text` token with just X (no backslash)
- **Whitespace-checking** — verifies no space after opening marker, no space before closing marker

## AST

```rust
pub struct MarkupDoc {
    pub children: Vec<MarkupNode>,
}

pub enum MarkupNode {
    Text(String),
    Formatted {
        style: MarkupStyle,
        children: Vec<MarkupNode>,
    },
}
```

The parser uses a **stack** to track open markers. It validates:
- Strict pairing (each close matches the most recent open of the same style)
- Non-empty content
- Whitespace rules
- Multiline vs single-line marker behavior

## MarkupStyle

Bitmask-based style set:

```rust
pub struct MarkupStyle(pub u32);

// Bits (currently 15 markers defined):
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

## Segmenter

Produces `DocumentCache` — a line-indexed cache:

```rust
pub struct DocumentCache {
    pub lines: Vec<MarkupCache>,
}

pub struct MarkupCache {
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub text: String,               // Rendered text (markers stripped)
    pub style: MarkupStyle,         // Style mask
    pub raw_start: usize,           // Byte offset in raw content
    pub raw_end: usize,
    pub disp_start: usize,          // Byte offset in display content
    pub disp_end: usize,
    pub open_len: usize,            // Length of opening marker
    pub close_len: usize,           // Length of closing marker
}
```

## Marker Definitions

Markers are defined in `ast.rs` as a static `MARKERS` array:

```rust
pub struct MarkerDef {
    pub open: &'static str,
    pub close: &'static str,
    pub style: MarkupStyle,
    pub multiline: bool,
}
```

### Current markers (14):

| Open | Close | Style | Multiline |
|------|-------|-------|-----------|
| `**` | `**` | BOLD | no |
| `//` | `//` | ITALIC | no |
| `__` | `__` | UNDERLINE | no |
| `~~` | `~~` | STRIKETHROUGH | no |
| `''` | `''` | SUPERSCRIPT | no |
| `,,` | `,,` | SUBSCRIPT | no |
| `` ` `` | `` ` `` | CODE | no |
| `==` | `==` | HIGHLIGHT | no |
| `!!` | `!!` | SPOILER | no |
| `++` | `++` | INSERTION | no |
| `--` | `--` | DELETION | no |
| `/*` | `*\` | COMMENT | **yes** |
| `$` | `$` | FORMULA | no |
| `$$` | `$$` | DISPLAY_FORMULA | **yes** |

Note: `!!!` (SPOILER_BLOCK) and `%%` (COMMENT variant via `track_depth: false`) are defined in the data but may not be fully wired through the parser.

## Tests

The parser has ~30 unit tests covering:
- Plain text segments
- Simple bold, italic, underline
- Nested bold+italic
- Escape sequences
- Unclosed markers (treated as text)
- Whitespace violations
- Multiline spoiler blocks
- Newline separation
