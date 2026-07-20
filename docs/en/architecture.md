# Architecture

## Layer Diagram

```
┌─────────────────────────────────────────────────────────┐
│                      Zol Binary                         │
│  main.rs (--iced flag dispatches to egui or Iced)       │
├─────────────────────────────────────────────────────────┤
│                    GUI Layer                             │
│  gui/ (egui::App / Iced::Application)                   │
│  ┌─────────────────┐  ┌──────────────────────────────┐  │
│  │  egui backend   │  │  Iced backend (experimental) │  │
│  │  app.rs, run.rs │  │  app_iced.rs, iced_editor.rs │  │
│  └────────┬────────┘  └──────────────┬───────────────┘  │
│           │                          │                   │
├───────────┼──────────────────────────┼───────────────────┤
│           ▼                          ▼                   │
│  ┌──────────────────────────────────────────────┐       │
│  │          Editor Core                           │       │
│  │  editor/ (GUI-independent)                    │       │
│  │  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │       │
│  │  │ Cursor   │  │ Layout   │  │ Render       │  │       │
│  │  │ cursor.rs│  │ compute/ │  │ shape/paint  │  │       │
│  │  └──────────┘  └──────────┘  └──────┬───────┘  │       │
│  │  ┌──────────┐  ┌──────────┐         │          │       │
│  │  │ Markup   │  │ Cache    │         │          │       │
│  │  │ mdplus   │→│ Document │         │          │       │
│  │  └──────────┘  └──────────┘         │          │       │
│  │  ┌──────────────────────────┐       │          │       │
│  │  │ Font (font.rs)           │←──────│          │       │
│  │  └──────────────────────────┘                │       │
│  └──────────────────────────────────────────────┘       │
├─────────────────────────────────────────────────────────┤
│                    API Layer                              │
│  api/ (public interface for Rhai plugins)                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ cursor   │  │ text     │  │ editor   │               │
│  └──────────┘  └──────────┘  └──────────┘               │
├─────────────────────────────────────────────────────────┤
│                    Rhai Layer                             │
│  rhai/ (theme engine, plugins)                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ engine/  │  │ plugins/ │  │ system/  │               │
│  │          │  │          │  │ languages│               │
│  └──────────┘  └──────────┘  └──────────┘               │
└─────────────────────────────────────────────────────────┘
```

## Data Flow (Frame Cycle)

### egui backend

```
frame start
  │
  ├─ EditorWidget::ui()
  │   ├─ handle_input() → api::{text,cursor}::* → dirty=true
  │   ├─ dirty || input?
  │   │   ├─ mdplus::parse_document() → DocumentCache
  │   │   └─ render::build() → ShapedDocument (cosmic-text Buffer)
  │   └─ render::paint() → egui painter (galley-like glyph drawing)
  │
  └─ frame end
```

### Iced backend

```
Event → IcedEditor::update()
  ├─ keyboard → modify content/cursor, dirty.set(true)
  └─ mouse → buffer.hit(), set cursor, request_redraw()

Frame → IcedEditor::draw()
  ├─ dirty? → render::build() with viewport height
  └─ fill_quad() for each glyph + cursor
```

## Module Dependencies

```
main.rs
  ├── gui::run         (egui entry)
  │     └── gui::app::ZolApp
  │           └── editor::editor_widget::EditorWidget
  │                 ├── editor::cursor
  │                 ├── editor::input
  │                 ├── editor::render (build + paint)
  │                 ├── editor::layout::compute
  │                 ├── editor::font
  │                 ├── editor::markup
  │                 ├── editor::cache
  │                 └── editor::state
  ├── gui::app_iced    (Iced entry)
  │     └── gui::iced_editor::IcedEditor
  │           └── editor::render (build)
  │               └── editor::font
  ├── api              (public API)
  │     ├── api::cursor
  │     ├── api::text
  │     └── api::editor
  └── mdplus           (markup parser)
        ├── mdplus::token
        ├── mdplus::parser
        ├── mdplus::ast
        └── mdplus::segmenter
```

## Concurrency Model

- **Single-threaded** — both egui and Iced run on the main thread.
- **Font singletons** — `FontSystem` and `SwashCache` are wrapped in `OnceLock<Mutex<...>>` for safe access, even though only the main thread currently uses them.
- **No async** — the editor has no async operations. File I/O (save) is synchronous.
