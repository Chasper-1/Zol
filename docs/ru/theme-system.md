# Система тем

Zol использует **Rhai**-скрипты для темизации. Файл темы `theme.rhai` в корне проекта загружается при старте.

## Формат файла темы

Тема — это Rhai-отображение (map) с секциями editor и text:

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

## Поддерживаемые типы значений

| Тип Rhai | Тип Rust | Пример |
|----------|----------|--------|
| `float` | `f32` | `16.0` |
| `rgba(r, g, b, a)` | `Rgba` | `rgba(39, 46, 51, 0.9)` |
| `string` | `String` | `"sans-serif"` |

## Структура EditorTheme

```rust
pub struct EditorTheme {
    pub text: TextTheme,
    pub editor: EditorSection,
}

pub struct TextTheme {
    pub size: f32,              // размер шрифта (по умолчанию: 16.0)
    pub color: Rgba,            // цвет текста
    pub font_family: Option<String>,
}

pub struct EditorSection {
    pub padding: f32,
    pub radius: f32,
    pub background: Rgba,
}
```

## Обработка ошибок

Ошибки компиляции или выполнения Rhai выводятся в stderr, используется тема по умолчанию:

```
[Zol] Ошибка компиляции темы Rhai: {ошибка}. Использую тему по умолчанию
[Zol] Ошибка выполнения темы Rhai: {ошибка}. Использую тему по умолчанию
[Zol] Ошибка парсинга цвета «editor.background»: {ошибка}
[Zol] Ошибка парсинга цвета «text.color»: {ошибка}
```

## Инфраструктура Handle/ThemeSystem (планируется)

Система тем построена на базе `Handle<T>` / `ThemeSystem`. Сейчас парсится фиксированное подмножество Rhai-значений. В будущих версиях:

- `Handle<T>` — типизированный доступ к значениям темы по строковому ключу
- `ThemeSystem` — хранит `HashMap<String, HandleValue>`, поддерживает `set`, `get`, `get_or_default`, `reset`
- Темы будут расширяемыми через Rhai-модули

## Значения по умолчанию

Если `theme.rhai` отсутствует, не парсится или конкретное поле не найдено:

| Поле | Значение по умолчанию |
|------|-----------------------|
| `editor.padding` | `10.0` |
| `editor.radius` | `16.0` |
| `editor.background` | `rgba(39, 46, 51, 0.9)` |
| `text.size` | `16.0` |
| `text.color` | `rgba(205, 214, 244, 1.0)` |
