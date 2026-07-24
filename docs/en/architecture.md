# Architecture

*Translation of the Russian original.*

## Layer Diagram

```
┌──────────────────────────────────────────────────┐
│               Zol Binary                         │
│  src/main.rs (5 lines, entry point only)         │
├──────────────────────────────────────────────────┤
│   crates/gui    — Iced backend                   │
│   ┌──────────────────────────────────────┐      │
│   │  app_iced.rs  (Iced::Application)    │      │
│   │  iced_editor/                        │      │
│   │  ├── inner/    (EditorInner)         │      │
│   │  ├── widget/   (IcedEditor, draw,    │      │
│   │  │             input, widget)        │      │
│   │  ├── nav/      (cursor_x, raw_at_x,  │      │
│   │  │             move_vertical)        │      │
│   │  └── scroll/   (ensure_visible,      │      │
│   │                layout_y)             │      │
│   └──────────────┬───────────────────────┘      │
│                  │                              │
├──────────────────┼──────────────────────────────┤
│                  ▼                              │
│   crates/api   — public API                     │
│   ┌──────────────────────────────────────┐      │
│   │  cursor, text, file, editor,         │      │
│   │  zoll, theme, doc                    │      │
│   └──────────────┬───────────────────────┘      │
│                  │                              │
├──────────────────┼──────────────────────────────┤
│                  ▼                              │
│   crates/editor — editor core                   │
│   ┌──────────────────────────────────────┐      │
│   │  cursor/    (grapheme, word,         │      │
│   │             movement, types)         │      │
│   │  font/      (FontSystem singleton)   │      │
│   │  layout/    (TextRun, line_runs)     │      │
│   │  render/    (shape, build,           │      │
│   │             shaped_doc)              │      │
│   │  markup/    (segmenter, parser)      │      │
│   │  cache/     (DocumentCache)          │      │
│   │  theme/     (EditorTheme, color,     │      │
│   │             handle, registry)        │      │
│   │  state.rs   (EditMode, Document)     │      │
│   │  utils/     (line helpers)           │      │
│   │  rhai/      (theme engine, plugins)  │      │
│   └──────┬───────────────────────────────┘      │
│          │                                      │
├──────────┼──────────────────────────────────────┤
│          ▼                                      │
│   crates/zoll — markup parser                   │
│   ┌──────────────────────────────────────┐      │
│   │  token/   (Tokenizer)                │      │
│   │  parser/  (stack-based AST)          │      │
│   │  ast/     (nodes, markers, style)    │      │
│   │  lib.rs   (parse_document)           │      │
│   └──────────────────────────────────────┘      │
└──────────────────────────────────────────────────┘
```

## Data Flow (Frame Cycle)

```
Event → IcedEditor::update()
  ├─ keyboard → api::{text,cursor} → dirty = true
  └─ mouse → buffer.hit() → request_redraw()

Frame → IcedEditor::draw()
  ├─ dirty? → zoll::parse_document()
  │         → layout::compute_line_runs()
  │         → render::shape_document() (cosmic-text Buffer)
  │         → viewport optimization (visible lines only)
  └─ render: fill_text() for background, glyphs, cursor
```

## Crate Dependencies

```
main.rs
  └── gui
        ├── api
        │     └── editor
        │           ├── zoll
        │           ├── layout, render, markup, cache
        │           ├── cursor, font, theme, utils, state
        │           └── rhai
        └── editor (via api)
              └── zoll
```

All dependencies go **downward**: gui → api → editor → zoll. No circular dependencies.

## Concurrency

- **Single-threaded** — Iced runs on the main thread.
- **Font singletons** — `OnceLock<Mutex<...>>` for safe access.
- **No async** — file I/O is synchronous.

## Handle system and pipeline

### Core idea

A handle is an **action**, not data. Calling a handle = requesting a change.
A handle takes input and returns nothing.

```
handle: fn(input) → void
```

Nobody "reads" state, nobody "syncs" anything.
A plugin doesn't ask "what color is the cursor?" — it says "set cursor to red".

### Three layers of handles

```
┌────────────────────────────────────────────┐
│  HANDLE REGISTRY (core)                   │
│  set_tab_width(4)                         │
│  set_cursor_color("#f00")                 │
│  toggle_preview()                         │
│  insert_text("hello")                     │
│  ...                                      │
└──────────┬──────────────┬─────────────────┘
           │              │
           ▼              ▼
┌──────────────────┐  ┌──────────────────────┐
│  RON layer       │  │  Rhai layer          │
│  (static)        │  │  (dynamic)           │
│                  │  │                      │
│ tab_width = 4 ───┼──┤ set_tab_width(4)     │
│ theme = "dark" ──┼──┤ set_theme("dark")    │
│                  │  │                      │
│read at startup   │  │on_key("Esc", ...)    │
│→ calls handles   │  │on_plugin_load(...)   │
└──────────────────┘  └──────────┬───────────┘
                                 │
                                 ▼
                        ┌──────────────────────┐
                        │  PLUGINS             │
                        │  (isolated           │
                        │   handle copies)     │
                        │                      │
                        │  set_cursor_color()  │
                        │  set_bg_color()      │
                        │  insert_text()       │
                        │  ...                 │
                        └──────────────────────┘
```

### RON layer

Static data read once at startup.
Each value becomes a handle call:

```ron
Config(
    tab_width: 4,
    font_size: 14,
    theme: "dark",
)
```
→ `set_tab_width(4)`, `set_font_size(14)`, `set_theme("dark")`.

RON is not needed at runtime — it's already expanded into handle calls.

### Rhai layer

Dynamic: hotkeys, macros, custom commands.
Calls the same handles as RON, but at runtime, in response to events.

```rhai
on_key("Escape", || { set_mode("Preview"); });
on_key("Ctrl-P", || { toggle_preview(); });
```

### Plugins

A plugin receives an isolated copy of the handle registry.
It cannot read/write core state directly — only call handles.
The core decides whether to apply the plugin's result.

This enables:
- **Sandboxing** — plugin doesn't touch core directly
- **Pipeline** — multiple plugins can process one input sequentially (each calls handles, core collects results)
- **Infinite nesting** — a plugin can load sub-plugins, each with its own registry copy

### RON vs Rhai

A specific handle can be available in both layers.
The difference is not in which handles exist, but in the nature of the call:

- **RON** — handle is called once at config load with a hardcoded value
- **Rhai** — handle is called repeatedly, on events, with computed values
- **Plugin** — handle is called from an isolated environment, result is validated by core

### Shared log (stderr-like channel)

**Problem:** a handle returns nothing. A plugin calls `set_theme("dark")` and has no idea what happened. If a plugin needs to react to changes (e.g. another plugin changed colors), there's no mechanism to find out. Subscriptions are complex, read-only handles break the concept.

**Solution:** append-only log that everyone writes to and anyone can read.

```
CORE:
  |  DEBUG | set_theme("dark") → OK
  |  DEBUG | set_tab_width(4) → OK
  |  DEBUG | PLUGIN_X: set_cursor_color("#fff")
  |  WARN  | PLUGIN_Y: remove_file("/etc/passwd") → REJECTED (sandbox)
  |  ERROR | PLUGIN_Z: insert_text() → "unexpected argument: color"

PLUGIN: reads last N entries:
  "theme changed to dark, I'll adjust my colors"
  "oh, plugin X changed the cursor — I'll do the same"
```

**How it works:**

- Every handle call is logged: who called it, with what args, what result (OK / REJECTED / ERROR).
- A plugin can read the log at any point. Not a subscription — it decides when to look.
- Core writes to the log too — the full picture is always visible.
- The log stores the last N entries (ring buffer) — old entries are pushed out.
- Nobody synchronizes anything. Append-only, everyone writes their own, nobody deletes others.

**Why this is better than events/subscriptions:**

- No runtime overhead for notifications — the log just exists, read it when you want
- Debugging built-in — you can see who called what, in what order, what went wrong
- One plugin can depend on another: it sees PLUGIN_X called `set_font_size(16)` in the log and adapts
- Core can analyze the log for anomalies: "this plugin calls the same handle 1000 times/sec" → throttle/ban
- To replay state, just replay the log from the start (event sourcing)
