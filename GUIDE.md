# GUIDE — Flint Editor

## 1. Архитектура

Проект разделён на три слоя:

```
src/api/         — публичные ручки для управления редактором (через них работают плагины)
src/editor/      — внутренняя реализация (Cursor, рендер, ввод)
src/gui/         — egui-обвязка (окно, панельки, кнопки)
src/mdplus/      — парсер разметки md+
```

`api/` — единственный способ взаимодействия с редактором снаружи. Внутренние модули (`editor/input.rs`) тоже ходят через `api/`, чтобы не дублировать логику.

---

## 2. EditorWidget — замена egui::TextEdit

`EditorWidget` — самописный виджет, который полностью контролирует:

- **Хранение текста** — `content: String`
- **Курсор** — `cursor: Cursor`
- **Кэш разметки** — `document_cache: DocumentCache` (результат mdplus-парсера)
- **Готовые для рисования строки** — `galleys: Galleys` (per-line `Arc<Galley>`)
- **Флаг dirty** — пересборка только при изменении текста

### Жизненный цикл кадра

```
ui() вызывается egui каждый кадр
  │
  ├─ Клик мыши → handle_click() → cursor.raw = позиция под курсором
  │
  ├─ handle_input() → клавиши → api::text::insert_at_cursor / api::cursor::move_*
  │
  ├─ dirty? → parse_document() + render::build() → новые galley
  │
  └─ render::paint() → рисует galley + курсор
```

### Repaint control

- **Preview** — `request_repaint_after(10s)` — почти мёртвый сон
- **Source / LivePreview** — `request_repaint_after(530ms)` — только для blink курсора

`parse_document` и `render::build` вызываются **только** когда текст реально изменился (dirty).

---

## 3. Cursor — навигация

Курсор оперирует **байтовой позицией** в полном `content` (`raw: usize`) и **номером строки** (`line: usize`).

Все движения — через `char_indices()`, никакой байтовой арифметики вручную.

### Доступные движения

| Метод | Куда |
|-------|------|
| `move_left` | Предыдущий символ (char boundary) |
| `move_right` | Следующий символ |
| `move_home` | Начало строки |
| `move_end` | Конец строки |
| `move_up` | Строка выше (col_visual anchor) |
| `move_down` | Строка ниже (col_visual anchor) |

### col_visual для вертикального движения

Когда курсор двигается вверх/вниз, он запоминает визуальную X-позицию (`col_visual: f32`). На новой строке он пытается встать на тот же X. Если строка короче — в конец строки.

`move_home` сбрасывает `col_visual` в 0.
`move_end` устанавливает `col_visual` в `f32::MAX` (будет вставать в конец любой строки).

---

## 4. Режимы отображения (EditMode)

| Режим | Рендер | Ввод | Маркеры видны |
|-------|--------|------|---------------|
| **Source** | Исходный текст с подсветкой | Да | Да (серым цветом) |
| **Preview** | Красивый markup (без маркеров) | Нет | Нет |
| **LivePreview** | Активная строка = Source, остальные = Preview | Да | Только на строке с курсором |

При переключении режима галлеи перестраиваются (dirty = true).

---

## 5. API — навигация (src/api/cursor.rs)

### Примитивы (строительные блоки)

```rust
pub fn prev_char(content: &str, from: usize) -> usize;
pub fn next_char(content: &str, from: usize) -> usize;
pub fn prev_word_start(content: &str, from: usize) -> usize;
pub fn next_word_start(content: &str, from: usize) -> usize;
pub fn next_word_end(content: &str, from: usize) -> usize;
```

Эти функции ничего не знают о редакторе — просто вычисляют позиции в строке. Из них можно собрать любое движение.

### Готовые движения

```rust
pub fn move_left(widget: &mut EditorWidget);
pub fn move_right(widget: &mut EditorWidget);
pub fn move_up(widget: &mut EditorWidget);
pub fn move_down(widget: &mut EditorWidget);
pub fn move_home(widget: &mut EditorWidget);
pub fn move_end(widget: &mut EditorWidget);
pub fn move_word_left(widget: &mut EditorWidget);   // Ctrl+←
pub fn move_word_right(widget: &mut EditorWidget);  // Ctrl+→
```

### Как собрать своё движение

```rust
use crate::api::cursor::{prev_char, next_char, line_start};

fn jump_to_prev_line_start(widget: &mut EditorWidget) {
    let line = widget.cursor.line;
    if line == 0 { return; }
    let start = line_start(&widget.content, line - 1);
    widget.cursor.raw = start;
    widget.cursor.line = line - 1;
    widget.cursor.reset_col_visual();
}
```

### Информационные ручки

```rust
pub fn cursor_pos(widget: &EditorWidget) -> usize;   // байтовая позиция
pub fn cursor_line(widget: &EditorWidget) -> usize;  // номер строки
```

---

## 6. API — редактирование (src/api/text.rs)

### Готовые операции

```rust
pub fn insert_at_cursor(widget: &mut EditorWidget, text: &str);
pub fn delete_before_cursor(widget: &mut EditorWidget);   // backspace
pub fn delete_after_cursor(widget: &mut EditorWidget);    // delete
pub fn newline(widget: &mut EditorWidget);
```

Каждая операция:
- Меняет `content`
- Двигает курсор
- Ставит `dirty = true` → вызовет перепарсинг и пересборку галлей на следующем кадре

### Чтение текста

```rust
pub fn get_text(widget: &EditorWidget) -> &str;
pub fn get_line(widget: &EditorWidget, idx: usize) -> Option<&str>;
pub fn get_line_count(widget: &EditorWidget) -> usize;
pub fn text_len(widget: &EditorWidget) -> usize;
```

---

## 7. API — редактор (src/api/editor.rs)

```rust
pub fn set_mode(state: &mut EditorState, mode: EditMode);
pub fn get_mode(state: &EditorState) -> EditMode;
```

---

## 8. Горячие клавиши (встроенные)

| Комбинация | Действие |
|-----------|----------|
| Печатные символы | Вставка под курсором |
| Backspace | Удалить символ перед курсором |
| Delete | Удалить символ после курсора |
| Enter | Перевод строки |
| ← → ↑ ↓ | Движение курсора |
| Home / End | Начало / конец строки |
| Ctrl+← / Ctrl+→ | В начало предыдущего / следующего слова |
| Ctrl+V | Вставка из буфера |
| Ctrl+S | Сохранение в notes.md |

---

## 9. Стили текста (markup)

Парсер md+ распознаёт маркеры и присваивает сегментам битовые флаги (`StyleFlags`). Рендер применяет к каждому стилю цвет/оформление:

| Флаг | Маркер | Визуал |
|------|--------|--------|
| `STYLE_BOLD` | `**` | Красный текст |
| `STYLE_ITALIC` | `//` | Голубой курсив |
| `STYLE_UNDERLINE` | `__` | Подчёркивание |
| `STYLE_STRIKETHROUGH` | `~~` | Зачёркнутый (серый) |
| `STYLE_SUPERSCRIPT` | `''` | Верхний индекс, зелёный |
| `STYLE_SUBSCRIPT` | `,,` | Нижний индекс, жёлтый |
| `STYLE_CODE` | `` ` `` | Моноширинный, серый |
| `STYLE_HIGHLIGHT` | `==` | Жёлтый фон |
| `STYLE_INSERTION` | `++` | Зелёный текст |
| `STYLE_DELETION` | `--` | Красный + зачёркнутый |
| `STYLE_COMMENT` | `/* */` | Серый курсив |
| `STYLE_FORMULA` | `$` | Моноширинный, зелёный |
| `STYLE_DISPLAY_FORMULA` | `$$` | Моноширинный + крупнее, зелёный |
| `STYLE_SPOILER` | `!!` / `!!!` | (ожидает реализации) |

Цвета — хардкод в `render.rs:segment_format()`. В будущем будут настраиваться через тему/rhai.

---

## 10. Как добавить новую горячую клавишу

Пример: добавить `Ctrl+W` для удаления слова перед курсором.

1. **input.rs** — добавить проверку в замыкание:
```rust
let pressed_ctrl_w = command && pressed(egui::Key::W);
```

2. **input.rs** — добавить вызов в основном теле:
```rust
if pressed_ctrl_w {
    api::text::delete_word_before_cursor(widget);
    dirty = true;
}
```

3. **api/text.rs** — реализовать ручку:
```rust
pub fn delete_word_before_cursor(widget: &mut EditorWidget) {
    let start = crate::api::cursor::prev_word_start(&widget.content, widget.cursor.raw);
    widget.content.drain(start..widget.cursor.raw);
    widget.cursor.raw = start;
    widget.cursor.update_line(&widget.content);
    widget.dirty = true;
}
```

---

## 11. Rhai-плагины (перспектива)

Когда будешь подключать Rhai:
- `src/rhai/api.rs` будет выборочно пробрасывать функции из `src/api/*`
- Плагины смогут двигать курсор, вставлять/удалять текст, менять режимы
- Ограничения: только те функции, которые явно зарегистрированы (безопасность)
