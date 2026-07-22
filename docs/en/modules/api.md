# API Module

*Translation of the Russian original.*

`crates/api/` — public interface for Rhai plugins and external consumers. All communication with the editor from outside goes through this crate.

```
api/
├── mod.rs
├── doc/        — Document creation, text reading
├── cursor/     — Movement primitives and compound commands
├── text/       — Text editing operations
├── file/       — File save/load
├── editor/     — Mode switching
├── zoll/       — zoll markup parsing
├── theme/      — Theme management
└── gui/        — GUI integration (iced)
```

## Cursor API (`api::cursor`)

### Movement

```rust
pub fn move_left(doc: &mut Document);
pub fn move_right(doc: &mut Document);
pub fn move_up(doc: &mut Document);
pub fn move_down(doc: &mut Document);
pub fn move_home(doc: &mut Document);
pub fn move_end(doc: &mut Document);
pub fn move_word_left(doc: &mut Document);    // Ctrl+←
pub fn move_word_right(doc: &mut Document);   // Ctrl+→
```

### Info

```rust
pub fn cursor_raw(doc: &Document) -> usize;   // byte offset
pub fn cursor_line(doc: &Document) -> usize;  // line number
pub fn cursor_col(doc: &Document) -> f32;     // visual column (pixels)
pub fn cursor_set_raw(doc: &mut Document, byte: usize);
pub fn cursor_set_line(doc: &mut Document, line: usize);
pub fn cursor_set_col(doc: &mut Document, col: f32);
pub fn cursor_reset_col(doc: &mut Document);
```

## Text API (`api::text`)

### Editing

```rust
pub fn insert_at_cursor(doc: &mut Document, text: &str);
pub fn delete_before(doc: &mut Document);   // Backspace
pub fn delete_after(doc: &mut Document);    // Delete
pub fn newline(doc: &mut Document);         // Enter
pub fn insert_at(doc: &mut Document, byte: usize, text: &str);
pub fn delete_range(doc: &mut Document, start: usize, end: usize);
```

Each operation:
1. Modifies `doc.content`
2. Updates cursor position
3. Sets `doc.dirty = true` (triggers re-parse and re-shape on next frame)

## Document API (`api::doc`)

```rust
pub fn doc_create(text: &str) -> Document;
pub fn doc_text(doc: &Document) -> &str;
pub fn doc_line(doc: &Document, idx: usize) -> Option<&str>;
pub fn doc_line_count(doc: &Document) -> usize;
pub fn doc_len(doc: &Document) -> usize;
pub fn doc_is_empty(doc: &Document) -> bool;
```

## File API (`api::file`)

```rust
pub fn file_save(doc: &Document, path: impl AsRef<Path>) -> io::Result<()>;
pub fn file_load(path: impl AsRef<Path>) -> io::Result<Document>;
pub fn file_save_str(text: &str, path: impl AsRef<Path>) -> io::Result<()>;
pub fn file_load_str(path: impl AsRef<Path>) -> io::Result<String>;
```

## Editor API (`api::editor`)

```rust
pub fn editor_set_mode(state: &mut EditorState, mode: EditMode);
pub fn editor_get_mode(state: &EditorState) -> EditMode;
pub fn editor_mode_name(mode: EditMode) -> &'static str;
pub fn editor_state_create(text: &str) -> EditorState;
```

## zoll API (`api::zoll`)

```rust
pub fn zoll_tokenize(text: &str) -> Vec<Token>;
pub fn zoll_parse(text: &str) -> MarkupDoc;
pub fn zoll_parse_cache(text: &str) -> DocumentCache;
```

## Theme API (`api::theme`)

```rust
pub fn theme_default() -> EditorTheme;
pub fn theme_set_name(theme: &mut EditorTheme, name: &str);
pub fn theme_get_name(theme: &EditorTheme) -> &str;
pub fn theme_set_bg(theme: &mut EditorTheme, hex: &str) -> Result<(), String>;
pub fn theme_set_text(theme: &mut EditorTheme, hex: &str) -> Result<(), String>;
```

## Rhai Integration (Planned)

When Rhai plugins are connected:

- `crates/editor/src/rhai/api.rs` will selectively expose functions from `crates/api/`
- Plugins can move cursor, insert/delete text, switch modes
- Safety: only explicitly registered functions are exposed