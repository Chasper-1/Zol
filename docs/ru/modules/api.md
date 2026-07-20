# Модуль API

`src/api/` — публичный интерфейс для Rhai-плагинов и внешних потребителей. Всё общение с редактором извне идёт через этот модуль.

```
api/
├── mod.rs
├── cursor.rs    — Примитивы движения и составные команды
├── text.rs      — Операции редактирования текста
└── editor.rs    — Переключение режимов
```

## API курсора (`api::cursor`)

### Примитивы

```rust
pub fn prev_char(content: &str, from: usize) -> usize;
pub fn next_char(content: &str, from: usize) -> usize;
pub fn prev_word_start(content: &str, from: usize) -> usize;
pub fn next_word_start(content: &str, from: usize) -> usize;
pub fn next_word_end(content: &str, from: usize) -> usize;
pub fn line_start(content: &str, line: usize) -> usize;
pub fn line_end(content: &str, line: usize) -> usize;
```

Эти функции работают с сырыми строками — они ничего не знают о редакторе и являются переиспользуемыми строительными блоками.

### Составные движения

```rust
pub fn move_left(widget: &mut EditorWidget);
pub fn move_right(widget: &mut EditorWidget);
pub fn move_up(widget: &mut EditorWidget);
pub fn move_down(widget: &mut EditorWidget);
pub fn move_home(widget: &mut EditorWidget);
pub fn move_end(widget: &mut EditorWidget);
pub fn move_word_left(widget: &mut EditorWidget);    // Ctrl+←
pub fn move_word_right(widget: &mut EditorWidget);   // Ctrl+→
```

### Информация

```rust
pub fn cursor_pos(widget: &EditorWidget) -> usize;   // байтовая позиция
pub fn cursor_line(widget: &EditorWidget) -> usize;  // номер строки
```

## API текста (`api::text`)

### Редактирование

```rust
pub fn insert_at_cursor(widget: &mut EditorWidget, text: &str);
pub fn delete_before_cursor(widget: &mut EditorWidget);   // Backspace
pub fn delete_after_cursor(widget: &mut EditorWidget);    // Delete
pub fn newline(widget: &mut EditorWidget);                // Enter
```

Каждая операция:
1. Изменяет `widget.content`
2. Обновляет позицию курсора
3. Устанавливает `widget.dirty = true` (вызывает перепарсинг и перешейп на следующем кадре)

### Чтение

```rust
pub fn get_text(widget: &EditorWidget) -> &str;
pub fn get_line(widget: &EditorWidget, idx: usize) -> Option<&str>;
pub fn get_line_count(widget: &EditorWidget) -> usize;
pub fn text_len(widget: &EditorWidget) -> usize;
```

## API редактора (`api::editor`)

```rust
pub fn set_mode(state: &mut EditorState, mode: EditMode);
pub fn get_mode(state: &EditorState) -> EditMode;
```

## Интеграция с Rhai (планируется)

Когда будут подключены Rhai-плагины:

- `src/rhai/api.rs` будет выборочно пробрасывать функции из `src/api/`
- Плагины смогут двигать курсор, вставлять/удалять текст, переключать режимы
- Безопасность: только явно зарегистрированные функции доступны

## Примечание

Составные движения `api::cursor` работают с `EditorWidget`. В Iced виджет напрямую вызывает методы `editor::cursor::Cursor`.
