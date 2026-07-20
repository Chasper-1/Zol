# Zol

**Zol** is a semantic text editor with custom markup (zml — Zol Markup Language), Iced rendering, Rhai-based theming, and grapheme-aware cursor navigation.

## Quick Start

```bash
cargo run
```

### Default file

Zol opens `notes.zml` in the project root. If the file doesn't exist, it is created empty.

### Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Home / End | Start / end of line |
| Ctrl+← / Ctrl+→ | Word left / right |
| Backspace / Delete | Delete character |
| Enter | New line |
| Ctrl+S | Save to `notes.zml` |

## Project Structure

```
src/
├── api/             — Public API for Rhai plugins
├── editor/          — Editor core (GUI-independent)
│   ├── cursor.rs    — Grapheme-aware cursor
│   ├── font/        — Font system (fontdb + cosmic-text)
│   ├── layout/      — TextRun computation from zml markup
│   ├── render/      — Shaping (cosmic-text) + rendering
│   ├── markup/      — zml parser integration
│   ├── cache/       — DocumentCache, MarkupCache, Segment
│   ├── state.rs     — EditorState, EditMode
│   ├── theme/       — EditorTheme, Rhai theme parser
│   └── utils/       — Line utilities
├── gui/             — Iced backend
│   ├── app_iced.rs  — Iced application
│   └── iced_editor.rs — Iced custom widget
├── zml/             — zml markup parser
└── main.rs          — Entry point
```

## Architecture

```
Event → IcedEditor::update()
  ├─ keyboard → modify content/cursor, dirty = true
  └─ mouse → buffer.hit(), set cursor, request_redraw()

Frame → IcedEditor::draw()
  ├─ dirty? → render::build() with viewport height
  └─ fill_quad() for each glyph + cursor
```

## Key Design Decisions

1. **Grapheme cursor** — `GraphemeCursor` for all navigation.
2. **zml markup** — single-pass tokenizer, stack-based AST, 15 style flags. File extension: `.zml`.
3. **Font system** — `OnceLock<Mutex<FontSystem>>` singleton via fontdb.
4. **Viewport shaping** — only visible lines are shaped via cosmic-text.
5. **Iced** — sole GUI backend, custom Widget via `fill_quad()`.
6. **Rhai theming** — `theme.rhai` loaded at startup.

## License

GNU General Public License v3.0.
