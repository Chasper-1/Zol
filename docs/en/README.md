# Zol

*Translation of the Russian original.*

**Zol** is a semantic text editor with custom markup (zoll — Zol Markup Language), Iced rendering, Rhai-based theming, and grapheme-aware cursor navigation.

## Quick Start

```bash
cargo run
```

### Default file

Zol opens `notes.zoll` in the project root. If the file doesn't exist, it is created empty.

### Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Home / End | Start / end of line |
| Ctrl+← / Ctrl+→ | Word left / right |
| Backspace / Delete | Delete character |
| Enter | New line |
| Ctrl+S | Save to `notes.zoll` |

## Project Structure

```
src/
├── api/             — Public API for Rhai plugins
│   ├── doc/         — Document creation, text reading
│   ├── cursor/      — Cursor movement and accessors
│   ├── text/        — Text editing operations
│   ├── file/        — File save/load
│   ├── editor/      — Editor mode switching
│   ├── zoll/        — zoll markup parsing
│   ├── theme/       — Theme management
│   └── gui/         — GUI integration (iced)
├── document.rs      — Document state (content + cursor + dirty flag)
├── editor/          — Editor core (GUI-independent)
│   ├── cursor.rs    — Grapheme-aware cursor
│   ├── font/        — Font system (fontdb + cosmic-text)
│   ├── layout/      — TextRun computation from zoll markup
│   ├── render/      — Shaping (cosmic-text) + rendering
│   ├── markup/      — zoll parser integration
│   ├── cache/       — DocumentCache, MarkupCache, Segment types
│   ├── state.rs     — EditorState, EditMode
│   ├── theme/       — EditorTheme, Rhai theme parser
│   └── utils/       — Line utilities
├── gui/             — Iced backend
│   ├── app_iced.rs  — Iced application
│   └── iced_editor/ — Iced custom widget
│       ├── inner.rs — Editor state (EditorInner)
│       ├── widget.rs — IcedEditor widget
│       ├── nav.rs   — Vertical navigation
│       └── scroll.rs — Auto-scroll
├── zoll/            — zoll markup parser
│   ├── mod.rs       — Public API: parse_document()
│   ├── ast.rs       — MarkupDoc, MarkupNode, MarkupStyle
│   ├── token.rs     — Tokenizer
│   ├── parser.rs    — Stack-based AST builder
│   └── segmenter.rs — AST → DocumentCache conversion
└── main.rs          — Entry point
```

## Architecture

```
Event → IcedEditor::update()
  ├─ keyboard → api::{text,cursor} → dirty = true
  └─ mouse → buffer.hit() → set cursor, request_redraw()

Frame → IcedEditor::draw()
  ├─ dirty? → render::build() with viewport height
  └─ fill_text() for each glyph + cursor
```

## Key Design Decisions

1. **Grapheme cursor** — `GraphemeCursor` for all navigation.
2. **zoll markup** — single-pass tokenizer, stack-based AST, 15 style flags. File extension: `.zoll`.
3. **Font system** — `OnceLock<Mutex<FontSystem>>` singleton via fontdb.
4. **Viewport shaping** — only visible lines are shaped via cosmic-text.
5. **Iced** — sole GUI backend, custom Widget via `fill_text()`.
6. **Rhai theming** — `theme.rhai` loaded at startup.

## License

GNU General Public License v3.0.