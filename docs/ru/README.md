# Zol

**Zol** — семантический текстовый редактор с собственной разметкой zml (Zol Markup Language), отрисовкой через Iced, темизацией через Rhai и навигацией по grapheme-кластерам.

## Быстрый старт

```bash
cargo run
```

### Файл по умолчанию

Zol открывает `notes.zml` в корне проекта. Если файла нет — создаётся пустой.

### Управление

| Клавиша | Действие |
|---------|----------|
| Стрелки | Движение курсора |
| Home / End | Начало / конец строки |
| Ctrl+← / Ctrl+→ | Слово влево / вправо |
| Backspace / Delete | Удалить символ |
| Enter | Новая строка |
| Ctrl+S | Сохранить в `notes.zml` |

## Структура проекта

```
src/
├── api/             — Публичное API для Rhai-плагинов
│   ├── cursor.rs    — Примитивы движения курсора
│   ├── text.rs      — Операции редактирования
│   └── editor.rs    — Переключение режимов
├── editor/          — Ядро редактора (независимо от GUI)
│   ├── cursor.rs    — Курсор с grapheme-навигацией
│   ├── font/        — Система шрифтов (fontdb + cosmic-text)
│   ├── layout/      — Вычисление TextRun из zml-разметки
│   ├── render/      — Шейпинг (cosmic-text Buffer) + отрисовка
│   ├── markup/      — Интеграция zml-парсера
│   ├── cache/       — DocumentCache, MarkupCache, Segment
│   ├── state.rs     — EditorState, EditMode
│   ├── editor_widget.rs — Iced-виджет редактора
│   ├── input.rs     — Обработка клавиатуры
│   ├── theme/       — EditorTheme, парсер темы Rhai
│   └── utils/       — Утилиты работы со строками
├── gui/             — Iced-бэкенд
│   ├── app_iced.rs  — Iced-приложение
│   └── iced_editor.rs — Iced-виджет (Widget trait)
├── zml/             — Парсер разметки zml
│   ├── token.rs     — Токенизатор (один проход)
│   ├── parser.rs    — Стековый построитель AST
│   ├── ast.rs       — MarkupDoc, MarkupNode, MarkupStyle
│   └── segmenter.rs — AST → DocumentCache
└── main.rs          — Точка входа
```

## Архитектура

```
Событие → IcedEditor::update()
  ├─ клавиатура → изменить content/cursor, dirty = true
  └─ мышь → buffer.hit(), установить курсор, request_redraw()

Кадр → IcedEditor::draw()
  ├─ dirty? → render::build() с высотой вьюпорта
  │     ├─ zml::parse_document() → DocumentCache
  │     └─ layout::compute_line_runs() → TextRun[] → shape_document()
  └─ fill_quad() для каждого глифа + курсор
```

## Ключевые решения

1. **Grapheme-курсор** — `Cursor` использует `GraphemeCursor` (unicode-segmentation) для всей навигации. Нет байтовой арифметики.

2. **Парсер разметки (zml)** — однопроходный токенизатор, стековый AST, 15 флагов стилей. Многострочные маркеры закрываются через переводы строк. Расширение файла — `.zml`.

3. **Система шрифтов** — синглтон `OnceLock<Mutex<FontSystem>>`, инициализируется один раз из системных шрифтов через fontdb.

4. **Viewport-шейпинг** — `render::build()` принимает `viewport_height`: cosmic-text шейпит только видимые строки.

5. **Iced** — единственный GUI-бэкенд. Кастомный виджет рисует через `fill_quad()`.

6. **Темизация Rhai** — `theme.rhai` загружается при старте.

## Лицензия

GNU General Public License v3.0.
