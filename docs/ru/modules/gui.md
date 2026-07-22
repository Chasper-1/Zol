# Модуль GUI

`src/gui/` — графический интерфейс на Iced.

```
gui/
├── mod.rs
├── app_iced.rs     — Iced-приложение
└── iced_editor/    — Пользовательский Iced-виджет
    ├── mod.rs      — Реэкспорты модулей
    ├── inner.rs    — Состояние редактора (EditorInner)
    ├── widget.rs   — Виджет IcedEditor (Widget trait)
    ├── nav.rs      — Вертикальная навигация (сохранение пиксельной X)
    └── scroll.rs   — Автоскролл курсора в видимую зону
```

## IcedEditor

`gui::iced_editor::IcedEditor<'a>` — пользовательский `iced::advanced::Widget`, рисующий через `fill_text()`.

### Внутреннее состояние

```rust
pub struct EditorInner {
    pub doc: RefCell<Document>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: RefCell<DocumentCache>,
    pub mode: EditMode,
    pub base_size: f32,
    pub heading_size: f32,
    pub theme: EditorTheme,
    pub scroll_y: Cell<f32>,
    pub file_path: String,
}
```

Interior mutability обеспечивается полями-`RefCell`. Виджет держит `&EditorInner`.

### Обработка событий (`update()`)

**Клавиатура:**
- Стрелки → навигация курсора
- Home / End → начало/конец строки
- Backspace / Delete → удаление символа
- Enter → новая строка
- Печатные символы → вставка текста
- Ctrl+S → сохранение файла
- Каждая мутация: `dirty.set(true)`

**Мышь:**
- Клик → `buffer.hit(local_x, local_y)` → конвертация в позицию курсора
- Колёсико → скролл

### Отрисовка (`draw()`)

Две фазы:

1. **Перешейп** (если dirty):
   - `render::build()` с `viewport_height = Some(bounds.height)`
   - Шейпятся только видимые строки

2. **Отрисовка:**
   - Фоновый quad
   - `fill_text()` для каждого layout_run
   - Курсор (полоса 2px, мигающая)

### Приложение

`app_iced.rs` — стандартный Iced boot/update/view:

```rust
fn boot() → (AppState, Task<Message>)
fn update(app_state: &mut AppState, message: Message)
fn view(app_state: &AppState) → Element<'_, Message, Theme, iced::Renderer>
```

`view` оборачивает `IcedEditor` в `Container`.

## Статус реализации

| Возможность | Статус |
|-------------|--------|
| Редактирование текста | ✅ |
| zoll-разметка | ✅ |
| Навигация курсора (влево/вправо/домой/конец) | ✅ |
| move_up / move_down | ✅ |
| Скролл (колёсико + автоскролл) | ✅ |
| Сохранение (Ctrl+S) | ✅ |
| Тема | ✅ |
| Позиционирование курсора мышью | ✅ |