# Font Module

*Translation of the Russian original.*

`crates/editor/src/font/` — global font system for Zol.

## Singleton Architecture

```rust
static GLOBAL: OnceLock<Mutex<FontGlobal>> = OnceLock::new();

struct FontGlobal {
    font_system: cosmic_text::FontSystem,
    swash_cache: cosmic_text::SwashCache,
}
```

- **`OnceLock`** — initialized exactly once, thread-safe (though Zol is single-threaded).
- **`Mutex`** — enables safe access from any thread, future-proof for potential multithreading (e.g., background shaping).

## API

```rust
// Initialize (idempotent — subsequent calls are no-ops)
pub fn init();

// Access FontSystem for shaping operations
pub fn with_font_system<F, T>(f: F) -> T
where F: FnOnce(&mut cosmic_text::FontSystem) -> T;

// Access SwashCache for glyph rasterization
pub fn with_swash_cache<F, T>(f: F) -> T
where F: FnOnce(&mut cosmic_text::SwashCache) -> T;

// Access FontSystem and SwashCache simultaneously (avoids deadlock)
pub fn with_font_and_cache<F, T>(f: F) -> T
where F: FnOnce(&mut cosmic_text::FontSystem, &mut cosmic_text::SwashCache) -> T;

// List all available font families
pub fn list_families() -> Vec<String>;

// Rescan system fonts (e.g., after installing a new font)
pub fn reload_system_fonts();
```

## Initialization

`init()` is called automatically by `render::build()` on its first invocation. It:

1. Creates a `fontdb::Database`
2. Calls `db.load_system_fonts()` — loads all fonts from the OS
3. Creates `cosmic_text::FontSystem` with locale "en" and the populated database
4. Creates an empty `cosmic_text::SwashCache`
5. Stores everything in the global `OnceLock`

## Usage in Rendering

```
render::build()
    │
    ├─ font::init()  (first call only)
    │
    └─ font::with_font_system(|fs| {
           shape_document(&runs, fs, base_size, font_family, viewport_height)
       })
```

`with_font_system` locks the global mutex and provides `&mut FontSystem` to the closure. The mutex is held only for the duration of the closure.

## Dependencies

- `cosmic-text` v0.19 — shaping engine
- `fontdb` (via cosmic-text re-export) — font database and discovery
- No other font crates (no `fontconfig`, `freetype`, or system libraries beyond what cosmic-text depends on)

## Planned

- Embedded fonts: load `.ttf` files from a bundled resources directory
- Font fallback configuration via theme
- Per-face selection (weight, style) instead of just family name
