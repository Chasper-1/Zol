# Zol

**Zol** — семантический текстовый редактор с собственной разметкой (md+), двумя бэкендами отрисовки (egui + Iced), темизацией через Rhai и навигацией по grapheme-кластерам.

## Быстрый старт

```bash
cargo run              # egui (по умолчанию)
cargo run -- --iced    # Iced (экспериментально)
```

### Файл по умолчанию

Zol открывает `notes.md` в корне проекта. Если файла нет — создаётся пустой.

### Управление

| Клавиша | Действие |
|---------|----------|
| Стрелки | Движение курсора |
| Home / End | Начало / конец строки |
| Ctrl+← / Ctrl+→ | Слово влево / вправо |
| Backspace / Delete | Удалить символ |
| Enter | Новая строка |
| Ctrl+S | Сохранить в `notes.md` |

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
│   ├── layout/      — Вычисление TextRun из md+ разметки
│   ├── render/      — Шейпинг (cosmic-text Buffer) + отрисовка
│   ├── markup/      — Интеграция md+ парсера
│   ├── cache/       — DocumentCache, MarkupCache, Segment
│   ├── state.rs     — EditorState, EditMode
│   ├── editor_widget.rs — egui-виджет редактора
│   ├── input.rs     — Обработка клавиатуры (egui)
│   ├── theme/       — EditorTheme, парсер темы Rhai
│   └── utils/       — Утилиты работы со строками
├── gui/             — GUI бэкенды
│   ├── app.rs       — egui-приложение (ZolApp)
│   ├── run.rs       — Точка входа egui, загрузка темы
│   ├── app_iced.rs  — Iced-приложение
│   └── iced_editor.rs — Iced-виджет (Widget trait)
├── mdplus/          — Парсер разметки md+
│   ├── token.rs     — Токенизатор (один проход)
│   ├── parser.rs    — Стековый построитель AST
│   ├── ast.rs       — MarkupDoc, MarkupNode, MarkupStyle
│   └── segmenter.rs — AST → DocumentCache
└── main.rs          — Точка входа (флаг --iced)
```

## Архитектура

```
notes.md ──→ gui::run ──→ EditorWidget::ui()
                              │
                    ┌─────────┴──────────┐
                    │                    │
              handle_input()       render::paint()
                    │                    │
                    ▼                    ▼
              api::text /           cosmic-text
              api::cursor           Buffer → GPU
                    │
                    ▼
              mdplus::parse_document()
                    │
                    ▼
              DocumentCache
                    │
                    ▼
              render::build()
                    │
                    ▼
              ShapedDocument
              (cosmic-text Buffer)
```

## Ключевые архитектурные решения

1. **Grapheme-курсор** — `Cursor` использует `GraphemeCursor` (unicode-segmentation) для всей навигации. Нет байтовой арифметики, нет O(n) линейных проходов.

2. **Парсер разметки (md+)** — однопроходный токенизатор, стековый AST, 14 флагов стилей. Многострочные маркеры (`/* */`, `$$ $$`, `!!! !!!`) закрываются через переводы строк.

3. **Система шрифтов** — синглтон `OnceLock<Mutex<FontSystem>>`, инициализируется один раз из системных шрифтов через fontdb.

4. **Viewport-шейпинг** — `render::build()` принимает `viewport_height`: если задана, cosmic-text шейпит только видимые строки.

5. **Два GUI** — egui (`EditorWidget`) стабильный бэкенд; Iced (`IcedEditor`) в разработке. Флаг `--iced` выбирает версию.

6. **Темизация Rhai** — `theme.rhai` загружается при старте. Парсится подмножество Rhai-значений в `EditorTheme`.

## Лицензия

GNU General Public License v3.0.
