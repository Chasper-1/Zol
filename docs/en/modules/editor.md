# Editor Module

`src/editor/` — core editor logic, independent of any GUI framework.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports all submodules |
| `cursor.rs` | Grapheme-aware cursor with vertical movement |
| `editor_widget.rs` | egui editor widget (legacy, being replaced by Iced) |
| `font/mod.rs` | Font system singleton (fontdb + cosmic-text) |
| `input.rs` | Keyboard input handling for egui backend |
| `layout/` | TextRun computation from md+ markup |
| `render/` | Shaping (cosmic-text Buffer) + painting |
| `state.rs` | EditorState, EditMode |
| `theme/` | EditorTheme, Rhai theme parser |
| `utils/` | Line boundary utilities, safe string slicing |
| `cache/` | DocumentCache, MarkupCache, Segment types |
| `markup/` | md+ parser integration (parse_document) |

## Cursor

`editor::cursor::Cursor` — position in the text, grapheme-aware.

```
Field        Type     Description
─────────────────────────────────────────
raw          usize    Byte offset (always on grapheme boundary)
line         usize    Cached line number
col_visual   f32      Horizontal anchor for move_up/move_down
last_blink   Instant  Last visibility change timestamp
```

### Navigation (all O(cluster) via GraphemeCursor)

| Method | Description |
|--------|-------------|
| `move_left` | Previous grapheme cluster |
| `move_right` | Next grapheme cluster |
| `move_home` | Start of line (resets col_visual) |
| `move_end` | End of line (sets col_visual to f32::MAX) |
| `move_up` | Line above, preserving col_visual |
| `move_down` | Line below, preserving col_visual |

### Blink

`should_blink()` — returns `true` for 530ms after the last cursor movement, then alternates every 530ms.

## Font System

`editor::font::` — singleton managing `cosmic_text::FontSystem` and `SwashCache`.

```
Global state: OnceLock<Mutex<FontGlobal>>
  FontGlobal {
    font_system: cosmic_text::FontSystem,
    swash_cache: cosmic_text::SwashCache,
  }
```

API:

| Function | Description |
|----------|-------------|
| `init()` | Initialize (safe to call multiple times) |
| `with_font_system(f)` | Access FontSystem for shaping |
| `with_swash_cache(f)` | Access SwashCache for rasterizing |
| `list_families()` | List all available font families |
| `reload_system_fonts()` | Rescan system fonts |

Fonts are loaded from the operating system via `fontdb::Database::load_system_fonts()`.

## Layout

`editor::layout::compute::compute_line_runs()` — converts a source line with its `MarkupCache` into `Vec<TextRun>`, applying style flags and emitting zero-width markers (shown gray in Source mode).

```
TextRun {
    text: String,
    byte_offset: usize,
    size: f32,
    font_family: Option<String>,
    color: Rgba,
}
```

## Render

`editor::render::` — two-phase pipeline:

### Phase 1: Build

`render::build()` → `ShapedDocument`

1. Split content into lines
2. For each line: `layout::compute::compute_line_runs()` → `Vec<TextRun>`
3. Merge runs into one `cosmic_text::Buffer` via `shape::shape_document()`
4. Buffer shapes text and stores glyph positions

Accepts `viewport_height: Option<f32>` — if set, only visible lines are shaped.

### Phase 2: Paint (egui)

`render::paint()` — iterates `buffer.layout_runs()`, draws each glyph as a colored quad via `egui::Painter`.

Also draws the cursor: a 2px-wide vertical bar at the cursor position, blinking.

`click_position()` — converts a mouse click pixel position to a byte offset using `buffer.hit()`.

## ShapedDocument

```rust
pub struct ShapedDocument {
    pub buffer: cosmic_text::Buffer,
}

impl ShapedDocument {
    pub fn total_height(&self) -> f32;
    pub fn line_count(&self) -> usize;
    pub fn line_height(&self, i: usize) -> f32;
}
```

## EditMode

```rust
pub enum EditMode {
    Preview,        // Render markup without markers, read-only
    LivePreview,    // Active line = Source, rest = Preview
    Source,         // Show raw markup with markers
}
```

## EditorState

```rust
pub struct EditorState {
    pub theme: EditorTheme,
    pub content: String,
    pub mode: EditMode,
}
```

## Data Flow

```
handle_input()
    │
    ▼
api::text::insert_at_cursor()  or  api::cursor::move_*()
    │
    ▼
dirty = true
    │
next frame ──► mdplus::parse_document() → DocumentCache
    │
    ▼
render::build() → ShapedDocument (cosmic-text Buffer)
    │
    ▼
render::paint() → screen
```
