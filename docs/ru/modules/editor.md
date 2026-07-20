# Модуль Editor

`src/editor/` — ядро редактора, независимое от GUI.

## Файлы

| Файл | Назначение |
|------|------------|
| `mod.rs` | Реэкспорт всех подмодулей |
| `cursor.rs` | Курсор с grapheme-навигацией и вертикальным движением |
| `editor_widget.rs` | Iced-виджет редактора (обёртка над IcedEditor) |
| `font/mod.rs` | Синглтон системы шрифтов (fontdb + cosmic-text) |
| `input.rs` | Обработка клавиатурного ввода |
| `layout/` | Вычисление TextRun из zml-разметки |
| `render/` | Шейпинг (cosmic-text Buffer) + отрисовка |
| `state.rs` | EditorState, EditMode |
| `theme/` | EditorTheme, парсер темы Rhai |
| `utils/` | Утилиты границ строк, безопасные слайсы |
| `cache/` | DocumentCache, MarkupCache, Segment |
| `markup/` | Интеграция zml-парсера (parse_document) |

## Курсор

`editor::cursor::Cursor` — позиция в тексте, grapheme-ориентированная.

```
Поле         Тип        Описание
───────────────────────────────────────────
raw          usize      Байтовый оффсет (всегда на границе grapheme)
line         usize      Кешированный номер строки
col_visual   f32        Горизонтальный якорь для move_up/move_down
last_blink   Instant    Время последнего изменения видимости
```

### Навигация (все O(cluster) через GraphemeCursor)

| Метод | Описание |
|--------|----------|
| `move_left` | Предыдущий grapheme-кластер |
| `move_right` | Следующий grapheme-кластер |
| `move_home` | Начало строки (сбрасывает col_visual) |
| `move_end` | Конец строки (устанавливает col_visual в f32::MAX) |
| `move_up` | Строка выше, сохраняя col_visual |
| `move_down` | Строка ниже, сохраняя col_visual |

### Мигание

`should_blink()` — возвращает `true` 530мс после последнего движения курсора, затем мигает с периодом 530мс.

## Система шрифтов

`editor::font::` — синглтон, управляющий `cosmic_text::FontSystem` и `SwashCache`.

```rust
static GLOBAL: OnceLock<Mutex<FontGlobal>> = OnceLock::new();
```

API:

| Функция | Описание |
|---------|----------|
| `init()` | Инициализация (безопасно вызывать многократно) |
| `with_font_system(f)` | Доступ к FontSystem для шейпинга |
| `with_swash_cache(f)` | Доступ к SwashCache для растрирования |
| `list_families()` | Список всех доступных семейств шрифтов |
| `reload_system_fonts()` | Пересканировать системные шрифты |

## Layout

`editor::layout::compute::compute_line_runs()` — преобразует строку исходного текста с её `MarkupCache` в `Vec<TextRun>`, применяя флаги стилей zml.

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

`editor::render::build()` — двухфазный пайплайн:

1. Разбить контент на строки
2. Для каждой строки: `layout::compute::compute_line_runs()` → `Vec<TextRun>`
3. `shape::shape_document()` — склеить runs в `cosmic_text::Buffer`
4. Buffer шейпит текст, хранит позиции глифов

Принимает `viewport_height: Option<f32>` — шейпятся только видимые строки.

Отрисовка (в Iced): `iced_editor.rs` итерирует `buffer.layout_runs()` и рисует glyph-quad'ы через `fill_quad()`.

## ShapedDocument

```rust
pub struct ShapedDocument {
    pub buffer: cosmic_text::Buffer,
}
```

## EditMode

```rust
pub enum EditMode {
    Preview,        // Показать разметку без маркеров, только чтение
    LivePreview,    // Активная строка = Source, остальные = Preview
    Source,         // Показать сырую разметку с маркерами
}
```

## Поток данных

```
Ввод → IcedEditor::update()
    │
    ├─ zml::parse_document() → DocumentCache
    │
    └─ dirty = true

Кадр → IcedEditor::draw()
    ├─ dirty? → render::build()
    └─ отрисовка глифов
```
