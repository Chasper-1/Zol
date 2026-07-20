# Editor Module

`src/editor/` — core editor logic, independent of any GUI framework.

## Files

| File | Purpose |
|------|---------|
| `cursor.rs` | Grapheme-aware cursor with vertical movement |
| `font/mod.rs` | Font system singleton (fontdb + cosmic-text) |
| `layout/` | TextRun computation from zml markup |
| `render/` | Shaping (cosmic-text Buffer) + rendering |
| `state.rs` | EditorState, EditMode |
| `theme/` | EditorTheme, Rhai theme parser |
| `cache/` | DocumentCache, MarkupCache, Segment types |
| `markup/` | zml parser integration |

## Cursor

Uses `GraphemeCursor` for all navigation. Fields: `raw` (byte offset), `line`, `col_visual` (for vertical movement), `last_blink`.

## Font System

`editor::font::` — `OnceLock<Mutex<FontGlobal>>` singleton. Loads system fonts via fontdb.

## Render

`render::build()` → `ShapedDocument`. Accepts `viewport_height` for visible-only shaping.

## EditMode

```rust
pub enum EditMode { Preview, LivePreview, Source }
```
