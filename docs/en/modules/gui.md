# GUI Module

`src/gui/` — graphical interface backends.

```
gui/
├── mod.rs
├── app.rs          — egui application (ZolApp)
├── run.rs          — egui entry point, Rhai theme loading
├── app_iced.rs     — Iced application
└── iced_editor.rs  — Iced custom Widget
```

## egui Backend

### ZolApp

`gui::app::ZolApp` — implements `eframe::App` trait.

```
struct ZolApp {
    state: EditorState,     // mode, theme, content
    editor: EditorWidget,   // cursor, cache, shaped_doc
}
```

Entry: `gui::run::run_app()` — creates eframe NativeOptions with title "Zol", loads theme from `theme.rhai`, creates `ZolApp`.

### EditorWidget

`editor::editor_widget::EditorWidget` — custom egui widget replacing `egui::TextEdit`.

```
struct EditorWidget {
    content: String,
    cursor: Cursor,
    document_cache: DocumentCache,
    shaped_doc: ShapedDocument,
    dirty: bool,
    last_active_line: usize,
}
```

Frame lifecycle in `EditorWidget::ui()`:

1. `handle_input()` — processes key events through `api::{text,cursor}`
2. If input or dirty:
   - `mdplus::parse_document()` → fresh cache
   - `render::build()` → fresh ShapedDocument
3. `render::paint()` — draws glyphs + cursor

### Repaint Strategy

- **Preview mode**: `request_repaint_after(Duration::from_secs(10))`
- **Source / LivePreview**: `request_repaint_after(Duration::from_millis(530))` (cursor blink)
- `parse_document` + `render::build` only run when content actually changed (dirty flag)

## Iced Backend

### IcedEditor (Widget)

`gui::iced_editor::IcedEditor<'a>` — custom `iced::advanced::Widget` that draws directly via `fill_quad()`.

```rust
pub struct EditorInner {
    pub content: RefCell<String>,
    pub cursor: RefCell<Cursor>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: DocumentCache,
    pub mode: EditMode,
    pub dirty: Cell<bool>,
    pub base_size: f32,
    pub heading_size: f32,
    pub theme: EditorTheme,
}
```

Interior mutability is provided by `RefCell` fields. The widget holds `&EditorInner` (shared reference).

### Event Handling

**Keyboard** (in `update()`):
- Arrow keys → cursor navigation
- Home / End → start/end of line
- Backspace / Delete → character deletion
- Enter → new line
- Printable chars → text insertion
- Each mutation sets `dirty.set(true)`

**Mouse**:
- Click → `buffer.hit(local_x, local_y)` → convert cosmic-text Cursor → Zol cursor position

### Rendering (in `draw()`)

Two-phase:

1. **Rebuild phase** (if dirty):
   - `render::build()` with `viewport_height = Some(bounds.height)`
   - Only visible lines are shaped

2. **Draw phase**:
   - Background quad
   - Glyph quads from `buffer.layout_runs()`
   - Cursor bar (2px wide, blinking)

### App

`gui::app_iced::` — standard Iced boot/update/view:

```rust
fn boot() → (AppState, Task<Message>)
fn update(app_state: &mut AppState, message: Message)
fn view(app_state: &AppState) → Element<'_, Message, Theme, iced::Renderer>
```

The view wraps `IcedEditor` in a `Scrollable` + `Container`.

## Future

The Iced backend is meant to replace egui entirely. Current status:

| Feature | egui | Iced |
|---------|------|------|
| Text editing | ✅ | ✅ |
| Cursor navigation | ✅ | ✅ (no up/down yet) |
| md+ rendering | ✅ | ✅ |
| Scroll | ✅ (egui native) | ❌ (TODO) |
| Save | ✅ (Ctrl+S) | ❌ (stub) |
| Theme | ✅ | ✅ |
| move_up/move_down | ✅ | ❌ (stub) |
