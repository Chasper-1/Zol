# Модуль Editor

`src/editor/` — ядро редактора, независимое от GUI-фреймворков.

## Файлы

| Файл | Назначение |
|------|------------|
| `mod.rs` | Реэкспорт всех подмодулей |
| `cursor.rs` | Курсор с grapheme-навигацией и вертикальным движением |
| `editor_widget.rs` | egui-виджет редактора (легаси, заменяется Iced) |
| `font/mod.rs` | Синглтон системы шрифтов (fontdb + cosmic-text) |
| `input.rs` | Обработка клавиатурного ввода для egui |
| `layout/` | Вычисление TextRun из md+ разметки |
| `render/` | Шейпинг (cosmic-text Buffer) + отрисовка |
| `state.rs` | EditorState, EditMode |
| `theme/` | EditorTheme, парсер темы Rhai |
| `utils/` | Утилиты границ строк, безопасные слайсы |
| `cache/` | DocumentCache, MarkupCache, Segment |
| `markup/` | Интеграция парсера md+ (parse_document) |

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

```
Глобальное состояние: OnceLock<Mutex<FontGlobal>>
  FontGlobal {
    font_system: cosmic_text::FontSystem,
    swash_cache: cosmic_text::SwashCache,
  }
```

API:

| Функция | Описание |
|---------|----------|
| `init()` | Инициализация (безопасно вызывать многократно) |
| `with_font_system(f)` | Доступ к FontSystem для шейпинга |
| `with_swash_cache(f)` | Доступ к SwashCache для растрирования |
| `list_families()` | Список всех доступных семейств шрифтов |
| `reload_system_fonts()` | Пересканировать системные шрифты |

Шрифты загружаются из операционной системы через `fontdb::Database::load_system_fonts()`.

## Layout

`editor::layout::compute::compute_line_runs()` — преобразует строку исходного текста с её `MarkupCache` в `Vec<TextRun>`, применяя флаги стилей и добавляя маркеры (серым цветом в режиме Source).

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

`editor::render::` — двухфазный пайплайн:

### Фаза 1: Build

`render::build()` → `ShapedDocument`

1. Разбить контент на строки
2. Для каждой строки: `layout::compute::compute_line_runs()` → `Vec<TextRun>`
3. Склеить все runs в один `cosmic_text::Buffer` через `shape::shape_document()`
4. Buffer шейпит текст и хранит позиции глифов

Принимает `viewport_height: Option<f32>` — если задана, шейпятся только видимые строки.

### Фаза 2: Paint (egui)

`render::paint()` — итерирует `buffer.layout_runs()`, рисует каждый глиф как цветной quad через `egui::Painter`.

Также рисует курсор: вертикальная полоса шириной 2px на позиции курсора, мигающая.

`click_position()` — конвертирует пиксельную позицию клика мыши в байтовый оффсет через `buffer.hit()`.

## ShapedDocument

```rust
pub struct ShapedDocument {
    pub buffer: cosmic_text::Buffer,
}

impl ShapedDocument {
    pub fn total_height(&self) -> f32;   // общая высота документа
    pub fn line_count(&self) -> usize;   // количество строк
    pub fn line_height(&self, i: usize) -> f32; // высота i-й строки
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

## EditorState

```rust
pub struct EditorState {
    pub theme: EditorTheme,
    pub content: String,
    pub mode: EditMode,
}
```

## Поток данных

```
handle_input()
    │
    ▼
api::text::insert_at_cursor()  или  api::cursor::move_*()
    │
    ▼
dirty = true
    │
следующий кадр ──► mdplus::parse_document() → DocumentCache
    │
    ▼
render::build() → ShapedDocument (cosmic-text Buffer)
    │
    ▼
render::paint() → экран
```
