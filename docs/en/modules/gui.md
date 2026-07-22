# GUI Module

*Translation of the Russian original.*

`src/gui/` — Iced graphical interface.

```
gui/
├── mod.rs
├── app_iced.rs     — Iced application
└── iced_editor/    — Iced custom widget
    ├── mod.rs      — Module re-exports
    ├── inner.rs    — Editor state (EditorInner)
    ├── widget.rs   — IcedEditor widget (Widget trait)
    ├── nav.rs      — Vertical navigation (pixel-X preservation)
    └── scroll.rs   — Auto-scroll cursor into view
```

## IcedEditor

`gui::iced_editor::IcedEditor<'a>` — custom `iced::advanced::Widget` rendering via `fill_text()`.

### EditorInner

```rust
pub struct EditorInner {
    pub doc: RefCell<Document>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: RefCell<DocumentCache>,
    pub mode: EditMode,
    pub base_size: f32,
    pub heading_size: f32,
    pub theme: EditorTheme,
    pub scroll_y: Cell<f32>,
    pub file_path: String,
}
```

Interior mutability via `RefCell` fields. The widget holds `&EditorInner`.

### Event Handling (`update()`)

**Keyboard:** cursor navigation, Home/End, Backspace/Delete, Enter, text input, Ctrl+S for save. Each mutation sets `dirty.set(true)`.

**Mouse:** click → `buffer.hit()` → cursor repositioning. Wheel → scroll.

### Rendering (`draw()`)

Two-phase:
1. If dirty: `render::build()` with viewport height (visible lines only)
2. Draw: background quad → `fill_text()` for each glyph → cursor bar (2px, blinking)

### Application

`app_iced.rs` — standard Iced boot/update/view, wraps IcedEditor in Container.

## Implementation Status

| Feature | Status |
|---------|--------|
| Text editing | ✅ |
| zoll markup | ✅ |
| Cursor navigation (left/right/home/end) | ✅ |
| move_up / move_down | ✅ |
| Scroll (wheel + auto-scroll) | ✅ |
| Save (Ctrl+S) | ✅ |
| Theme | ✅ |
| Mouse click positioning | ✅ |