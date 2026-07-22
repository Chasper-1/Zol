# Модуль API

`crates/api/` — публичный интерфейс для Rhai-плагинов и внешних потребителей. Всё общение с редактором извне идёт через этот крейт.

```
api/
├── mod.rs
├── doc/        — Создание документа, чтение текста
├── cursor/     — Примитивы движения и составные команды
├── text/       — Операции редактирования текста
├── file/       — Сохранение и загрузка файлов
├── editor/     — Переключение режимов
├── zoll/       — Парсинг zoll-разметки
└── theme/      — Управление темами
```

## API курсора (`api::cursor`)

### Движение

```rust
pub fn move_left(doc: &mut Document);
pub fn move_right(doc: &mut Document);
pub fn move_up(doc: &mut Document);
pub fn move_down(doc: &mut Document);
pub fn move_home(doc: &mut Document);
pub fn move_end(doc: &mut Document);
pub fn move_word_left(doc: &mut Document);    // Ctrl+←
pub fn move_word_right(doc: &mut Document);   // Ctrl+→
```

### Информация

```rust
pub fn cursor_raw(doc: &Document) -> usize;   // байтовая позиция
pub fn cursor_line(doc: &Document) -> usize;  // номер строки
pub fn cursor_col(doc: &Document) -> f32;     // визуальная колонка (пиксели)
pub fn cursor_set_raw(doc: &mut Document, byte: usize);
pub fn cursor_set_line(doc: &mut Document, line: usize);
pub fn cursor_set_col(doc: &mut Document, col: f32);
pub fn cursor_reset_col(doc: &mut Document);
```

## API текста (`api::text`)

### Редактирование

```rust
pub fn insert_at_cursor(doc: &mut Document, text: &str);
pub fn delete_before(doc: &mut Document);   // Backspace
pub fn delete_after(doc: &mut Document);    // Delete
pub fn newline(doc: &mut Document);         // Enter
pub fn insert_at(doc: &mut Document, byte: usize, text: &str);
pub fn delete_range(doc: &mut Document, start: usize, end: usize);
```

Каждая операция:
1. Изменяет `doc.content`
2. Обновляет позицию курсора
3. Устанавливает `doc.dirty = true` (вызывает перепарсинг и перешейп на следующем кадре)

## API документа (`api::doc`)

```rust
pub fn doc_create(text: &str) -> Document;
pub fn doc_text(doc: &Document) -> &str;
pub fn doc_line(doc: &Document, idx: usize) -> Option<&str>;
pub fn doc_line_count(doc: &Document) -> usize;
pub fn doc_len(doc: &Document) -> usize;
pub fn doc_is_empty(doc: &Document) -> bool;
```

## API файлов (`api::file`)

```rust
pub fn file_save(doc: &Document, path: impl AsRef<Path>) -> io::Result<()>;
pub fn file_load(path: impl AsRef<Path>) -> io::Result<Document>;
pub fn file_save_str(text: &str, path: impl AsRef<Path>) -> io::Result<()>;
pub fn file_load_str(path: impl AsRef<Path>) -> io::Result<String>;
```

## API редактора (`api::editor`)

```rust
pub fn editor_set_mode(state: &mut EditorState, mode: EditMode);
pub fn editor_get_mode(state: &EditorState) -> EditMode;
pub fn editor_mode_name(mode: EditMode) -> &'static str;
pub fn editor_state_create(text: &str) -> EditorState;
```

## API zoll (`api::zoll`)

```rust
pub fn zoll_tokenize(text: &str) -> Vec<Token>;
pub fn zoll_parse(text: &str) -> MarkupDoc;
pub fn zoll_parse_cache(text: &str) -> DocumentCache;
```

## API темы (`api::theme`)

```rust
pub fn theme_default() -> EditorTheme;
pub fn theme_set_name(theme: &mut EditorTheme, name: &str);
pub fn theme_get_name(theme: &EditorTheme) -> &str;
pub fn theme_set_bg(theme: &mut EditorTheme, hex: &str) -> Result<(), String>;
pub fn theme_set_text(theme: &mut EditorTheme, hex: &str) -> Result<(), String>;
```

## Интеграция с Rhai (планируется)

Когда будут подключены Rhai-плагины:

- `crates/editor/src/rhai/api.rs` будет выборочно пробрасывать функции из `crates/api/`
- Плагины смогут двигать курсор, вставлять/удалять текст, переключать режимы
- Безопасность: только явно зарегистрированные функции доступны