# zoll Parser Module

*Translation of the Russian original.*

`crates/zoll/` — standalone parser for the **zoll** markup language (Zol Markup Language).

```
zoll/
├── mod.rs        — Public API: parse_document()
├── ast.rs        — AST types (MarkupDoc, MarkupNode, MarkupStyle)
├── token.rs      — Tokenizer
├── parser.rs     — Stack-based AST builder
└── segmenter.rs  — AST → DocumentCache conversion
```

## Pipeline

```
Raw text (.zoll) → token::tokenize() → parser::parse() → segmenter::to_document_cache()
```

## Token Types

```rust
pub enum Token {
    Text(String),       // Plain text
    Open(MarkupStyle),  // Opening marker
    Close(MarkupStyle), // Closing marker
    Newline,            // Line break
}
```

Tokenizer: single pass O(n), no backtracking, escape-aware.

## AST

```rust
pub struct MarkupDoc { pub children: Vec<MarkupNode> }
pub enum MarkupNode {
    Text(String),
    Formatted { style: MarkupStyle, children: Vec<MarkupNode> },
}
```

Parser uses a stack for open markers. Validates pairing, nesting, whitespace rules.

## Markers (14)

| Open | Close | Style | Multiline |
|------|-------|-------|-----------|
| `**` | `**` | BOLD | no |
| `//` | `//` | ITALIC | no |
| `__` | `__` | UNDERLINE | no |
| `~~` | `~~` | STRIKETHROUGH | no |
| `''` | `''` | SUPERSCRIPT | no |
| `,,` | `,,` | SUBSCRIPT | no |
| `==` | `==` | HIGHLIGHT | no |
| `!!` | `!!` | SPOILER | no |
| `!!!` | `!!!` | SPOILER_BLOCK | **yes** |
| `++` | `++` | INSERTION | no |
| `--` | `--` | DELETION | no |
| `%%` | `%%` | COMMENT | **yes** |
| `$` | `$` | FORMULA | no |
| `$$` | `$$` | DISPLAY_FORMULA | **yes** |

`MarkupStyle`: u32 bitmask with 15 flags (14 markers + PLAIN). ~30 unit tests.