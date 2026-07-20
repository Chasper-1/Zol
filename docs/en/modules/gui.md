# GUI Module

`src/gui/` — Iced graphical interface.

```
gui/
├── mod.rs
├── app_iced.rs     — Iced application
└── iced_editor.rs  — Iced custom widget
```

## IcedEditor

`gui::iced_editor::IcedEditor<'a>` — custom `iced::advanced::Widget` rendering via `fill_quad()`.

### EditorInner

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

Interior mutability via `RefCell` fields. The widget holds `&EditorInner`.

### Event Handling (`update()`)

**Keyboard:** cursor navigation, Home/End, Backspace/Delete, Enter, text input. Each mutation sets `dirty.set(true)`.

**Mouse:** click → `buffer.hit()` → cursor repositioning.

### Rendering (`draw()`)

Two-phase:
1. If dirty: `render::build()` with viewport height (visible lines only)
2. Draw: background quad → glyph quads → cursor bar (2px, blinking)

### Application

`app_iced.rs` — standard Iced boot/update/view, wraps IcedEditor in Scrollable + Container.

## Implementation Status

| Feature | Status |
|---------|--------|
| Text editing | ✅ |
| zml markup | ✅ |
| Cursor navigation (left/right/home/end) | ✅ |
| move_up / move_down | ❌ (TODO) |
| Scroll | ❌ (TODO) |
| Save (Ctrl+S) | ❌ (stub) |
| Theme | ✅ |
