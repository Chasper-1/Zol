# Модуль GUI

`src/gui/` — графические интерфейсы.

```
gui/
├── mod.rs
├── app.rs          — egui-приложение (ZolApp)
├── run.rs          — Точка входа egui, загрузка темы Rhai
├── app_iced.rs     — Iced-приложение
└── iced_editor.rs  — Пользовательский Iced-виджет
```

## egui бэкенд

### ZolApp

`gui::app::ZolApp` — реализует `eframe::App`.

```
struct ZolApp {
    state: EditorState,     // режим, тема, контент
    editor: EditorWidget,   // курсор, кеш, shaped_doc
}
```

Вход: `gui::run::run_app()` — создаёт eframe NativeOptions с заголовком "Zol", загружает тему из `theme.rhai`, создаёт `ZolApp`.

### EditorWidget

`editor::editor_widget::EditorWidget` — пользовательский egui-виджет, заменяющий `egui::TextEdit`.

```
struct EditorWidget {
    content: String,
    cursor: Cursor,
    document_cache: DocumentCache,
    shaped_doc: ShapedDocument,
    dirty: bool,
    last_active_line: usize,
}
```

Жизненный цикл кадра в `EditorWidget::ui()`:

1. `handle_input()` — обрабатывает клавиши через `api::{text,cursor}`
2. Если был ввод или dirty:
   - `mdplus::parse_document()` → свежий кеш
   - `render::build()` → свежий ShapedDocument
3. `render::paint()` — рисует глифы + курсор

### Стратегия repaint

- **Preview**: `request_repaint_after(Duration::from_secs(10))`
- **Source / LivePreview**: `request_repaint_after(Duration::from_millis(530))` (мигание курсора)
- `parse_document` + `render::build` выполняются **только** когда текст реально изменился (флаг dirty)

## Iced бэкенд

### IcedEditor (Widget)

`gui::iced_editor::IcedEditor<'a>` — пользовательский `iced::advanced::Widget`, рисующий через `fill_quad()`.

```rust
pub struct EditorInner {
    pub content: RefCell<String>,
    pub cursor: RefCell<Cursor>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: DocumentCache,
    pub mode: EditMode,
    pub dirty: Cell<bool>,
    pub base_size: f32,
    pub heading_size: f32,
    pub theme: EditorTheme,
}
```

Interior mutability обеспечивается полями-`RefCell`. Виджет держит `&EditorInner` (разделяемая ссылка).

### Обработка событий

**Клавиатура** (в `update()`):
- Стрелки → навигация курсора
- Home / End → начало/конец строки
- Backspace / Delete → удаление символа
- Enter → новая строка
- Печатные символы → вставка текста
- Каждая мутация устанавливает `dirty.set(true)`

**Мышь**:
- Клик → `buffer.hit(local_x, local_y)` → конвертация cosmic-text Cursor в позицию Zol-курсора

### Отрисовка (в `draw()`)

Две фазы:

1. **Перешейп** (если dirty):
   - `render::build()` с `viewport_height = Some(bounds.height)`
   - Шейпятся только видимые строки

2. **Отрисовка**:
   - Фоновый quad
   - Glyph-quad'ы из `buffer.layout_runs()`
   - Курсор (полоса 2px, мигающая)

### Приложение

`gui::app_iced::` — стандартный Iced boot/update/view:

```rust
fn boot() → (AppState, Task<Message>)
fn update(app_state: &mut AppState, message: Message)
fn view(app_state: &AppState) → Element<'_, Message, Theme, iced::Renderer>
```

view оборачивает `IcedEditor` в `Scrollable` + `Container`.

## Планы

Iced-бэкенд должен полностью заменить egui. Текущий статус:

| Возможность | egui | Iced |
|-------------|------|------|
| Редактирование текста | ✅ | ✅ |
| Навигация курсора | ✅ | ✅ (без up/down) |
| md+ разметка | ✅ | ✅ |
| Скролл | ✅ (egui native) | ❌ (TODO) |
| Сохранение | ✅ (Ctrl+S) | ❌ (заглушка) |
| Тема | ✅ | ✅ |
| move_up/move_down | ✅ | ❌ (заглушка) |
