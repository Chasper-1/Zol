# Architecture

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
│  │  app_iced.rs, iced_editor.rs │            │
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
│  │  │ zml      │  │ Cache    │     │     │    │
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
  ├─ dirty? → zml::parse_document()
  │         → layout::compute_line_runs()
  │         → render::shape_document() (cosmic-text Buffer)
  │         → viewport optimization (visible lines only)
  └─ render: fill_quad() for background, glyphs, cursor
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
  └── zml (token, parser, ast, segmenter)
```

## Concurrency

- **Single-threaded** — Iced runs on the main thread.
- **Font singletons** — `OnceLock<Mutex<...>>` for safe access.
- **No async** — file I/O is synchronous.
