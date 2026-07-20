# Theme System

*Translation of the Russian original.*

Zol uses **Rhai** scripts for theming. The theme file `theme.rhai` in the project root is loaded at startup.

## Theme File Format

The theme is a Rhai map with editor and text sections:

```rhai
#{
    editor: #{
        padding: 10.0,
        radius: 16.0,
        background: rgba(39, 46, 51, 0.9),
    },
    text: #{
        size: 16.0,
        color: rgba(205, 214, 244, 1.0),
    }
}
```

## Supported Value Types

| Rhai type | Rust type | Example |
|-----------|-----------|---------|
| `float` | `f32` | `16.0` |
| `rgba(r, g, b, a)` | `Rgba` | `rgba(39, 46, 51, 0.9)` |
| `string` | `String` | `"sans-serif"` |

## EditorTheme Structure

```rust
pub struct EditorTheme {
    pub text: TextTheme,
    pub editor: EditorSection,
}

pub struct TextTheme {
    pub size: f32,         // font size (default: 16.0)
    pub color: Rgba,       // text color
    pub font_family: Option<String>,
}

pub struct EditorSection {
    pub padding: f32,
    pub radius: f32,
    pub background: Rgba,
}
```

## Error Handling

Errors during Rhai compilation or execution are printed to stderr, and a default theme is used as fallback:

```
[Zol] Theme compilation error: {error}. Using default theme.
[Zol] Theme execution error: {error}. Using default theme.
[Zol] Color parsing error for «editor.background»: {error}
[Zol] Color parsing error for «text.color»: {error}
```

## Handle/ThemeSystem Infrastructure (Planned)

The theme system is built on `Handle<T>` / `ThemeSystem`. Currently only a fixed subset of Rhai values is parsed. Future versions will support:

- `Handle<T>` — typed access to theme values via string keys
- `ThemeSystem` — stores `HashMap<String, HandleValue>`, supports `set`, `get`, `get_or_default`, `reset`
- Extensible themes via Rhai modules

## Default Values

If `theme.rhai` is missing, fails to parse, or a specific field is absent:

| Field | Default |
|-------|---------|
| `editor.padding` | `10.0` |
| `editor.radius` | `16.0` |
| `editor.background` | `rgba(39, 46, 51, 0.9)` |
| `text.size` | `16.0` |
| `text.color` | `rgba(205, 214, 244, 1.0)` |
