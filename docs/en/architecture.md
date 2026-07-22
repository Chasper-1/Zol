# Architecture

*Translation of the Russian original.*

## Layer Diagram

```
┌──────────────────────────────────────────────┐
│              Zol Binary                      │
│  main.rs                                     │
├──────────────────────────────────────────────┤
│              GUI Layer                       │
│  gui/ (Iced::Application)                    │
│  ┌──────────────────────────────┐            │
│  │  Iced backend                │            │
│  │  app_iced.rs, iced_editor/   │            │
│  │  ├── inner.rs                │            │
│  │  ├── widget.rs               │            │
│  │  ├── nav.rs                  │            │
│  │  └── scroll.rs               │            │
│  └──────────────┬───────────────┘            │
│                 │                            │
├─────────────────┼────────────────────────────┤
│                 ▼                            │
│  ┌──────────────────────────────────────┐    │
│  │          Editor Core                 │    │
│  │  editor/ (GUI-independent)           │    │
│  │  ┌──────────┐  ┌──────────┐  ┌────┐  │    │
│  │  │ Cursor   │  │ Layout   │  │Render│  │    │
│  │  │ cursor.rs│  │ compute/ │  │shape/│  │    │
│  │  └──────────┘  └──────────┘  └──┬──┘  │    │
│  │  ┌──────────┐  ┌──────────┐     │     │    │
│  │  │ zoll     │  │ Cache    │     │     │    │
│  │  │ parser   │→│ Document │     │     │    │
│  │  └──────────┘  └──────────┘     │     │    │
│  │  ┌──────────────────────┐       │     │    │
│  │  │ Font (font.rs)       │←──────│     │    │
│  │  └──────────────────────┘            │    │
│  └──────────────────────────────────────┘    │
├──────────────────────────────────────────────┤
│              API Layer                       │
│  api/ (public interface)                     │
├──────────────────────────────────────────────┤
│              Rhai Layer                      │
│  rhai/ (theme engine, plugins)               │
└──────────────────────────────────────────────┘
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

## Module Dependencies

```
main.rs
  ├── gui::app_iced
  │     └── gui::iced_editor::IcedEditor
  │           └── editor::render::build
  │                 ├── editor::font
  │                 └── editor::layout::compute
  ├── editor::cache
  ├── editor::cursor
  ├── editor::state
  ├── editor::theme
  ├── api::cursor, api::text, api::editor
  └── zoll (token, parser, ast, segmenter)
```

## Concurrency

- **Single-threaded** — Iced runs on the main thread.
- **Font singletons** — `OnceLock<Mutex<...>>` for safe access.
- **No async** — file I/O is synchronous.