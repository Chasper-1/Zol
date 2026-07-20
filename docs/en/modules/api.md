# API Module

`src/api/` — public interface for Rhai plugins and external consumers. All communication with the editor from outside goes through this module.

```
api/
├── mod.rs
├── cursor.rs    — Movement primitives and compound commands
├── text.rs      — Text editing operations
└── editor.rs    — Mode switching
```

## Cursor API (`api::cursor`)

### Primitives

```rust
pub fn prev_char(content: &str, from: usize) -> usize;
pub fn next_char(content: &str, from: usize) -> usize;
pub fn prev_word_start(content: &str, from: usize) -> usize;
pub fn next_word_start(content: &str, from: usize) -> usize;
pub fn next_word_end(content: &str, from: usize) -> usize;
pub fn line_start(content: &str, line: usize) -> usize;
pub fn line_end(content: &str, line: usize) -> usize;
```

These functions operate on raw strings — they don't know about the editor and are reusable building blocks.

### Compound Movements

```rust
pub fn move_left(widget: &mut EditorWidget);
pub fn move_right(widget: &mut EditorWidget);
pub fn move_up(widget: &mut EditorWidget);
pub fn move_down(widget: &mut EditorWidget);
pub fn move_home(widget: &mut EditorWidget);
pub fn move_end(widget: &mut EditorWidget);
pub fn move_word_left(widget: &mut EditorWidget);    // Ctrl+←
pub fn move_word_right(widget: &mut EditorWidget);   // Ctrl+→
```

### Info

```rust
pub fn cursor_pos(widget: &EditorWidget) -> usize;   // byte offset
pub fn cursor_line(widget: &EditorWidget) -> usize;  // line number
```

## Text API (`api::text`)

### Editing

```rust
pub fn insert_at_cursor(widget: &mut EditorWidget, text: &str);
pub fn delete_before_cursor(widget: &mut EditorWidget);   // Backspace
pub fn delete_after_cursor(widget: &mut EditorWidget);    // Delete
pub fn newline(widget: &mut EditorWidget);                // Enter
```

Each operation:
1. Modifies `widget.content`
2. Updates cursor position
3. Sets `widget.dirty = true` (triggers re-parse and re-shape on next frame)

### Reading

```rust
pub fn get_text(widget: &EditorWidget) -> &str;
pub fn get_line(widget: &EditorWidget, idx: usize) -> Option<&str>;
pub fn get_line_count(widget: &EditorWidget) -> usize;
pub fn text_len(widget: &EditorWidget) -> usize;
```

## Editor API (`api::editor`)

```rust
pub fn set_mode(state: &mut EditorState, mode: EditMode);
pub fn get_mode(state: &EditorState) -> EditMode;
```

## Rhai Integration (Planned)

When Rhai plugins are connected:

- `src/rhai/api.rs` will selectively expose functions from `src/api/`
- Plugins can move cursor, insert/delete text, switch modes
- Safety: only explicitly registered functions are exposed

## Note

The `api::cursor` compound movements operate on `EditorWidget`. For Iced, the widget directly calls `editor::cursor::Cursor` methods.
