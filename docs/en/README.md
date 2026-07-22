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
Zol/
  Cargo.toml              ← workspace (5 crates)
  crates/
    zoll/                 ← zoll markup parser (standalone)
      src/
        token/            ← Tokenizer (single pass)
        parser/           ← Stack-based AST builder
        ast/              ← AST nodes, markers, styles
        lib.rs            ← Public API: parse_document()

    editor/               ← Editor core (GUI-independent)
      src/
        cursor/           ← Grapheme-aware cursor
        font/             ← Font system (fontdb + cosmic-text)
        layout/           ← TextRun computation from zoll markup
        render/           ← Shaping (cosmic-text Buffer) + rendering
        markup/           ← zoll parser integration
        cache/            ← DocumentCache, MarkupCache, Segment
        state.rs          ← EditorState, EditMode
        theme/            ← EditorTheme, Rhai theme parser
        utils/            ← Line utilities
        rhai/             ← Rhai engine (themes, plugins)

    api/                  ← Public API for Rhai plugins
      src/
        doc/              ← Document creation, text reading
        cursor/           ← Cursor movement and accessors
        text/             ← Text editing operations
        file/             ← File save/load
        editor/           ← Editor mode switching
        zoll/             ← zoll markup parsing
        theme/            ← Theme management

    gui/                  ← Iced backend
      src/
        app_iced.rs       ← Iced application
        iced_editor/      ← Iced custom widget
          inner/          ← Editor state (EditorInner)
            data.rs
            edit_doc.rs
            mode.rs
          widget/         ← IcedEditor widget
            editor.rs
            widget.rs
            draw/         ← Rendering (background, text, cursor)
            input/        ← Input handling (keyboard, mouse)
          nav/            ← Vertical navigation
            cursor_x.rs
            raw_at_x.rs
            move_vertical.rs
          scroll/         ← Auto-scroll
            layout_y.rs
            ensure_visible.rs

  src/
    main.rs               ← Entry point (5 lines)
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
