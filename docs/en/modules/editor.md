# Editor Module

*Translation of the Russian original.*

`crates/editor/` — core editor logic, independent of any GUI framework.

## Files

| File | Purpose |
|------|---------|
| `cursor.rs` | Grapheme-aware cursor with vertical movement |
| `font/mod.rs` | Font system singleton (fontdb + cosmic-text) |
| `layout/` | TextRun computation from zoll markup |
| `render/` | Shaping (cosmic-text Buffer) + rendering |
| `state.rs` | EditorState, EditMode |
| `theme/` | EditorTheme, Rhai theme parser, color parser |
| `cache/` | DocumentCache, MarkupCache, Segment types |
| `markup/` | zoll parser integration (delegates to `zoll::parse_document`) |
| `utils/` | Line utilities (safe slice, line bounds, line counting) |

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