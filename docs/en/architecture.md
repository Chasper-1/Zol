# Architecture

*Translation of the Russian original.*

## Layer Diagram

```
┌──────────────────────────────────────────────────┐
│               Zol Binary                         │
│  src/main.rs (5 lines, entry point only)         │
├──────────────────────────────────────────────────┤
│   crates/gui    — Iced backend                   │
│   ┌──────────────────────────────────────┐      │
│   │  app_iced.rs  (Iced::Application)    │      │
│   │  iced_editor/                        │      │
│   │  ├── inner/    (EditorInner)         │      │
│   │  ├── widget/   (IcedEditor, draw,    │      │
│   │  │             input, widget)        │      │
│   │  ├── nav/      (cursor_x, raw_at_x,  │      │
│   │  │             move_vertical)        │      │
│   │  └── scroll/   (ensure_visible,      │      │
│   │                layout_y)             │      │
│   └──────────────┬───────────────────────┘      │
│                  │                              │
├──────────────────┼──────────────────────────────┤
│                  ▼                              │
│   crates/api   — public API                     │
│   ┌──────────────────────────────────────┐      │
│   │  cursor, text, file, editor,         │      │
│   │  zoll, theme, doc                    │      │
│   └──────────────┬───────────────────────┘      │
│                  │                              │
├──────────────────┼──────────────────────────────┤
│                  ▼                              │
│   crates/editor — editor core                   │
│   ┌──────────────────────────────────────┐      │
│   │  cursor/    (grapheme, word,         │      │
│   │             movement, types)         │      │
│   │  font/      (FontSystem singleton)   │      │
│   │  layout/    (TextRun, line_runs)     │      │
│   │  render/    (shape, build,           │      │
│   │             shaped_doc)              │      │
│   │  markup/    (segmenter, parser)      │      │
│   │  cache/     (DocumentCache)          │      │
│   │  theme/     (EditorTheme, color,     │      │
│   │             handle, registry)        │      │
│   │  state.rs   (EditMode, Document)     │      │
│   │  utils/     (line helpers)           │      │
│   │  rhai/      (theme engine, plugins)  │      │
│   └──────┬───────────────────────────────┘      │
│          │                                      │
├──────────┼──────────────────────────────────────┤
│          ▼                                      │
│   crates/zoll — markup parser                   │
│   ┌──────────────────────────────────────┐      │
│   │  token/   (Tokenizer)                │      │
│   │  parser/  (stack-based AST)          │      │
│   │  ast/     (nodes, markers, style)    │      │
│   │  lib.rs   (parse_document)           │      │
│   └──────────────────────────────────────┘      │
└──────────────────────────────────────────────────┘
```

## Data Flow (Frame Cycle)

```
Event → IcedEditor::update()
  ├─ keyboard → api::{text,cursor} → dirty = true
  └─ mouse → buffer.hit() → request_redraw()

Frame → IcedEditor::draw()
  ├─ dirty? → zoll::parse_document()
  │         → layout::compute_line_runs()
  │         → render::shape_document() (cosmic-text Buffer)
  │         → viewport optimization (visible lines only)
  └─ render: fill_text() for background, glyphs, cursor
```

## Crate Dependencies

```
main.rs
  └── gui
        ├── api
        │     └── editor
        │           ├── zoll
        │           ├── layout, render, markup, cache
        │           ├── cursor, font, theme, utils, state
        │           └── rhai
        └── editor (via api)
              └── zoll
```

All dependencies go **downward**: gui → api → editor → zoll. No circular dependencies.

## Concurrency

- **Single-threaded** — Iced runs on the main thread.
- **Font singletons** — `OnceLock<Mutex<...>>` for safe access.
- **No async** — file I/O is synchronous.
