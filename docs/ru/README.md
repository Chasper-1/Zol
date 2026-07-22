# Zol

**Zol** — семантический текстовый редактор с собственной разметкой zoll (Zol Markup Language), отрисовкой через Iced, темизацией через Rhai и навигацией по grapheme-кластерам.

## Быстрый старт

```bash
cargo run
```

### Файл по умолчанию

Zol открывает `notes.zoll` в корне проекта. Если файла нет — создаётся пустой.

### Управление

| Клавиша | Действие |
|---------|----------|
| Стрелки | Движение курсора |
| Home / End | Начало / конец строки |
| Ctrl+← / Ctrl+→ | Слово влево / вправо |
| Backspace / Delete | Удалить символ |
| Enter | Новая строка |
| Ctrl+S | Сохранить в `notes.zoll` |

## Структура проекта

```
src/
├── api/             — Публичное API для Rhai-плагинов
│   ├── doc/         — Создание документа, чтение текста
│   ├── cursor/      — Примитивы движения курсора
│   ├── text/        — Операции редактирования
│   ├── file/        — Сохранение и загрузка файлов
│   ├── editor/      — Переключение режимов
│   ├── zoll/        — Парсинг zoll-разметки
│   ├── theme/       — Управление темами
│   └── gui/         — GUI-ручки (Iced)
├── document.rs      — Состояние документа (контент + курсор + dirty-флаг)
├── editor/          — Ядро редактора (независимо от GUI)
│   ├── cursor.rs    — Курсор с grapheme-навигацией
│   ├── font/        — Система шрифтов (fontdb + cosmic-text)
│   ├── layout/      — Вычисление TextRun из zoll-разметки
│   ├── render/      — Шейпинг (cosmic-text Buffer) + отрисовка
│   ├── markup/      — Интеграция zoll-парсера
│   ├── cache/       — DocumentCache, MarkupCache, Segment
│   ├── state.rs     — EditorState, EditMode
│   ├── theme/       — EditorTheme, парсер темы Rhai
│   └── utils/       — Утилиты работы со строками
├── gui/             — Iced-бэкенд
│   ├── app_iced.rs  — Iced-приложение
│   └── iced_editor/ — Iced-виджет
│       ├── inner.rs — Состояние редактора (EditorInner)
│       ├── widget.rs — Виджет IcedEditor
│       ├── nav.rs   — Вертикальная навигация
│       └── scroll.rs — Автоскролл
├── zoll/            — Парсер разметки zoll
│   ├── mod.rs       — Публичное API: parse_document()
│   ├── ast.rs       — MarkupDoc, MarkupNode, MarkupStyle
│   ├── token.rs     — Токенизатор (один проход)
│   ├── parser.rs    — Стековый построитель AST
│   └── segmenter.rs — AST → DocumentCache
└── main.rs          — Точка входа
```

## Архитектура

```
Событие → IcedEditor::update()
  ├─ клавиатура → api::{text,cursor} → dirty = true
  └─ мышь → buffer.hit() → установить курсор, request_redraw()

Кадр → IcedEditor::draw()
  ├─ dirty? → render::build() с высотой вьюпорта
  └─ fill_text() для каждого глифа + курсор
```

## Ключевые решения

1. **Grapheme-курсор** — `Cursor` использует `GraphemeCursor` (unicode-segmentation) для всей навигации. Нет байтовой арифметики.

2. **Парсер разметки (zoll)** — однопроходный токенизатор, стековый AST, 15 флагов стилей. Многострочные маркеры закрываются через переводы строк. Расширение файла — `.zoll`.

3. **Система шрифтов** — синглтон `OnceLock<Mutex<FontSystem>>`, инициализируется один раз из системных шрифтов через fontdb.

4. **Viewport-шейпинг** — `render::build()` принимает `viewport_height`: cosmic-text шейпит только видимые строки.

5. **Iced** — единственный GUI-бэкенд. Кастомный виджет рисует через `fill_text()`.

6. **Темизация Rhai** — `theme.rhai` загружается при старте.

## Лицензия

GNU General Public License v3.0.