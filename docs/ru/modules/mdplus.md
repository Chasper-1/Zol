# Модуль парсера md+

`src/mdplus/` — автономный парсер разметки языка **md+**.

```
mdplus/
├── mod.rs        — Публичное API: parse_document()
├── ast.rs        — Типы AST (MarkupDoc, MarkupNode, MarkupStyle)
├── token.rs      — Токенизатор
├── parser.rs     — Стековый построитель AST
└── segmenter.rs  — Преобразование AST → DocumentCache
```

## Пайплайн

```
Сырой текст
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

Токенизатор:
- **Один проход** — O(n), без возвратов
- **Обрабатывает escape** — `\X` даёт `Text`-токен с одним X (без обратного слеша)
- **Проверяет пробелы** — после открывающего маркера не должно быть пробела, перед закрывающим — не должно

## AST

```rust
pub struct MarkupDoc {
    pub children: Vec<MarkupNode>,
}

pub enum MarkupNode {
    Text(String),
    Formatted {
        style: MarkupStyle,
        children: Vec<MarkupNode>,
    },
}
```

Парсер использует **стек** для отслеживания открытых маркеров. Валидирует:
- Строгую парность (каждый закрывающий маркер соответствует последнему открытому того же стиля)
- Непустой контент
- Правила пробелов
- Многострочность vs однострочность

## MarkupStyle

Набор стилей в виде битовой маски:

```rust
pub struct MarkupStyle(pub u32);

// Биты (сейчас 15 маркеров):
PLAIN           = 0
BOLD            = 1 << 0
ITALIC          = 1 << 1
UNDERLINE       = 1 << 2
STRIKETHROUGH   = 1 << 3
SUPERSCRIPT     = 1 << 4
SUBSCRIPT       = 1 << 5
CODE            = 1 << 6
HIGHLIGHT       = 1 << 7
SPOILER         = 1 << 8
SPOILER_BLOCK   = 1 << 9
INSERTION       = 1 << 10
DELETION        = 1 << 11
COMMENT         = 1 << 12
FORMULA         = 1 << 13
DISPLAY_FORMULA = 1 << 14
```

## Сегментатор

Производит `DocumentCache` — построчный кеш:

```rust
pub struct DocumentCache {
    pub lines: Vec<MarkupCache>,
}

pub struct MarkupCache {
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub text: String,               // Отображаемый текст (маркеры удалены)
    pub style: MarkupStyle,         // Маска стиля
    pub raw_start: usize,           // Байтовое смещение в сыром тексте
    pub raw_end: usize,
    pub disp_start: usize,          // Байтовое смещение в отображаемом тексте
    pub disp_end: usize,
    pub open_len: usize,            // Длина открывающего маркера
    pub close_len: usize,           // Длина закрывающего маркера
}
```

## Определения маркеров

Маркеры заданы в `ast.rs` как статический массив `MARKERS`:

```rust
pub struct MarkerDef {
    pub open: &'static str,
    pub close: &'static str,
    pub style: MarkupStyle,
    pub multiline: bool,
}
```

### Текущие маркеры (14):

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
| `++` | `++` | INSERTION | нет |
| `--` | `--` | DELETION | нет |
| `/*` | `*\` | COMMENT | **да** |
| `$` | `$` | FORMULA | нет |
| `$$` | `$$` | DISPLAY_FORMULA | **да** |

Примечание: `!!!` (SPOILER_BLOCK) и `%%` (вариант комментария через `track_depth: false`) определены в данных, но могут быть не полностью подключены через парсер.

## Тесты

Парсер имеет ~30 модульных тестов, покрывающих:
- Сегменты обычного текста
- Простой жирный, курсив, подчёркнутый
- Вложенные жирный+курсив
- Escape-последовательности
- Незакрытые маркеры (трактуются как текст)
- Нарушения пробельных правил
- Многострочные спойлер-блоки
- Разделение переводами строк
