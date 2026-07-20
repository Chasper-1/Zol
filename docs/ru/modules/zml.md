# Модуль парсера zml

`src/zml/` — автономный парсер разметки языка **zml** (Zol Markup Language).

```
zml/
├── mod.rs        — Публичное API: parse_document()
├── ast.rs        — Типы AST (MarkupDoc, MarkupNode, MarkupStyle)
├── token.rs      — Токенизатор
├── parser.rs     — Стековый построитель AST
└── segmenter.rs  — Преобразование AST → DocumentCache
```

## Пайплайн

```
Сырой текст (.zml)
    │
    ▼
token::tokenize(text)  →  Vec<Token>
    │
    ▼
parser::parse(&tokens)  →  MarkupDoc (AST)
    │
    ▼
segmenter::to_document_cache(&ast)  →  DocumentCache
    │
    ▼
editor::layout::compute_line_runs()  →  Vec<TextRun>
```

## Типы токенов

```rust
pub enum Token {
    Text(String),                   // Фрагмент обычного текста
    Open(MarkupStyle),              // Открывающий маркер
    Close(MarkupStyle),             // Закрывающий маркер
    Newline,                        // Перевод строки
}
```

Токенизатор: один проход O(n), без возвратов, обрабатывает escape.

## AST

```rust
pub struct MarkupDoc { pub children: Vec<MarkupNode> }

pub enum MarkupNode {
    Text(String),
    Formatted { style: MarkupStyle, children: Vec<MarkupNode> },
}
```

Парсер использует стек для отслеживания открытых маркеров.

## Маркеры zml (15)

| Открытие | Закрытие | Стиль | Многострочный |
|----------|----------|-------|---------------|
| `**` | `**` | BOLD | нет |
| `//` | `//` | ITALIC | нет |
| `__` | `__` | UNDERLINE | нет |
| `~~` | `~~` | STRIKETHROUGH | нет |
| `''` | `''` | SUPERSCRIPT | нет |
| `,,` | `,,` | SUBSCRIPT | нет |
| `` ` `` | `` ` `` | CODE | нет |
| `==` | `==` | HIGHLIGHT | нет |
| `!!` | `!!` | SPOILER | нет |
| `!!!` | `!!!` | SPOILER_BLOCK | **да** |
| `++` | `++` | INSERTION | нет |
| `--` | `--` | DELETION | нет |
| `/*` | `*\` | COMMENT | **да** |
| `$` | `$` | FORMULA | нет |
| `$$` | `$$` | DISPLAY_FORMULA | **да** |

Флаги стилей: `MarkupStyle` — битовая маска u32, 15 флагов. Тесты: ~30 unit-тестов.
