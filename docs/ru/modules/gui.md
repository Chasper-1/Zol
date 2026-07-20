# Модуль GUI

`src/gui/` — графический интерфейс на Iced.

```
gui/
├── mod.rs
├── app_iced.rs     — Iced-приложение
└── iced_editor.rs  — Пользовательский Iced-виджет
```

## IcedEditor

`gui::iced_editor::IcedEditor<'a>` — пользовательский `iced::advanced::Widget`, рисующий через `fill_quad()`.

### Внутреннее состояние

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

Interior mutability обеспечивается полями-`RefCell`. Виджет держит `&EditorInner`.

### Обработка событий (`update()`)

**Клавиатура:**
- Стрелки → навигация курсора
- Home / End → начало/конец строки
- Backspace / Delete → удаление символа
- Enter → новая строка
- Печатные символы → вставка текста
- Каждая мутация: `dirty.set(true)`

**Мышь:**
- Клик → `buffer.hit(local_x, local_y)` → конвертация в позицию курсора

### Отрисовка (`draw()`)

Две фазы:

1. **Перешейп** (если dirty):
   - `render::build()` с `viewport_height = Some(bounds.height)`
   - Шейпятся только видимые строки

2. **Отрисовка:**
   - Фоновый quad
   - Glyph-quad'ы из `buffer.layout_runs()`
   - Курсор (полоса 2px, мигающая)

### Приложение

`app_iced.rs` — стандартный Iced boot/update/view:

```rust
fn boot() → (AppState, Task<Message>)
fn update(app_state: &mut AppState, message: Message)
fn view(app_state: &AppState) → Element<'_, Message, Theme, iced::Renderer>
```

`view` оборачивает `IcedEditor` в `Scrollable` + `Container`.

## Статус реализации

| Возможность | Статус |
|-------------|--------|
| Редактирование текста | ✅ |
| zml-разметка | ✅ |
| Навигация курсора (влево/вправо/домой/конец) | ✅ |
| move_up / move_down | ❌ (TODO) |
| Скролл | ❌ (TODO) |
| Сохранение (Ctrl+S) | ❌ (заглушка) |
| Тема | ✅ |
