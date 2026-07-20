# Zol

**Zol** is a semantic text editor with custom markup (md+), dual rendering pipeline (egui + Iced), Rhai-based theming, and grapheme-aware cursor navigation.

## Quick Start

```bash
cargo run              # egui backend (default)
cargo run -- --iced    # Iced backend (experimental)
```

### Default file

Zol opens `notes.md` in the project root. If the file doesn't exist, it is created empty.

### Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Home / End | Start / end of line |
| Ctrl+← / Ctrl+→ | Word left / right |
| Backspace / Delete | Delete character |
| Enter | New line |
| Ctrl+S | Save to `notes.md` |

## Project Structure

```
src/
├── api/             — Public API for Rhai plugins
│   ├── cursor.rs    — Cursor movement primitives
│   ├── text.rs      — Text editing operations
│   └── editor.rs    — Mode switching, editor state
├── editor/          — Editor core (GUI-independent)
│   ├── cursor.rs    — Grapheme-aware cursor (GraphemeCursor)
│   ├── font/        — Font system (fontdb + cosmic-text)
│   ├── layout/      — TextRun computation from md+ markup
│   ├── render/      — Shaping (cosmic-text Buffer) + egui painting
│   ├── markup/      — md+ parser integration → DocumentCache
│   ├── cache/       — DocumentCache, MarkupCache, Segment
│   ├── state.rs     — EditorState, EditMode
│   ├── editor_widget.rs — egui editor widget
│   ├── input.rs     — Keyboard input handling (egui)
│   ├── theme/       — EditorTheme, Rhai theme parser
│   └── utils/       — Line utilities, safe slicing
├── gui/             — GUI backends
│   ├── app.rs       — egui application (ZolApp)
│   ├── run.rs       — egui entry point, Rhai theme loading
│   ├── app_iced.rs  — Iced application
│   └── iced_editor.rs — Iced custom widget (Widget trait)
├── mdplus/          — md+ markup parser
│   ├── token.rs     — Tokenizer (one pass, no recursion)
│   ├── parser.rs    — Stack-based AST builder
│   ├── ast.rs       — MarkupDoc, MarkupNode, MarkupStyle
│   └── segmenter.rs — AST → DocumentCache conversion
└── main.rs          — Entry point (--iced flag)
```

## Architecture

```
notes.md ──→ gui::run ──→ EditorWidget::ui()
                              │
                    ┌─────────┴──────────┐
                    │                    │
              handle_input()       render::paint()
                    │                    │
                    ▼                    ▼
              api::text /           cosmic-text
              api::cursor           Buffer → GPU
                    │
                    ▼
              mdplus::parse_document()
                    │
                    ▼
              DocumentCache
                    │
                    ▼
              render::build()
                    │
                    ▼
              ShapedDocument
              (cosmic-text Buffer)
```

## Key Design Decisions

1. **Grapheme cursor** — `Cursor` uses `GraphemeCursor` (unicode-segmentation) for all navigation. No byte arithmetic, no O(n) linear scans.

2. **Markup parser (md+)** — one-pass tokenizer, stack-based AST, 14 style flags. Multiline markers (`/* */`, `$$ $$`, `!!! !!!`) close across newlines.

3. **Font system** — `OnceLock<Mutex<FontSystem>>` singleton initialized once from system fonts via fontdb.

4. **Viewport shaping** — `render::build()` accepts `viewport_height`: if set, cosmic-text only shapes visible lines (`buffer.set_size(None, height)`).

5. **Dual GUI** — egui (`EditorWidget`) is the stable backend; Iced (`IcedEditor`) is in development. `--iced` flag controls which runs.

6. **Rhai theming** — `theme.rhai` is loaded at startup. Editor parses a subset of Rhai values (float, rgba, string) into `EditorTheme`.

## Licensing

GNU General Public License v3.0.
