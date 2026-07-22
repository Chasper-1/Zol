# Архитектура

## Диаграмма слоёв

```
┌──────────────────────────────────────────────────┐
│               Бинарник Zol                       │
│  src/main.rs (5 строк, только точка входа)       │
├──────────────────────────────────────────────────┤
│   crates/gui    — Iced-бэкенд                   │
│   ┌──────────────────────────────────────┐      │
│   │  app_iced.rs  (Iced::Application)    │      │
│   │  iced_editor/                        │      │
│   │  ├── inner/    (EditorInner)         │      │
│   │  ├── widget/   (IcedEditor, draw,    │      │
│   │  │             input, widget)        │      │
│   │  ├── nav/      (cursor_x, raw_at_x,  │      │
│   │  │             move_vertical)        │      │
│   │  └── scroll/   (ensure_visible,      │      │
│   │                layout_y)             │      │
│   └──────────────┬───────────────────────┘      │
│                  │                              │
├──────────────────┼──────────────────────────────┤
│                  ▼                              │
│   crates/api   — публичное API                  │
│   ┌──────────────────────────────────────┐      │
│   │  cursor, text, file, editor,         │      │
│   │  zoll, theme, doc                    │      │
│   └──────────────┬───────────────────────┘      │
│                  │                              │
├──────────────────┼──────────────────────────────┤
│                  ▼                              │
│   crates/editor — ядро редактора                │
│   ┌──────────────────────────────────────┐      │
│   │  cursor/    (grapheme, word,         │      │
│   │             movement, types)         │      │
│   │  font/      (FontSystem singleton)   │      │
│   │  layout/    (TextRun, line_runs)     │      │
│   │  render/    (shape, build,           │      │
│   │             shaped_doc)              │      │
│   │  markup/    (segmenter, parser)      │      │
│   │  cache/     (DocumentCache)          │      │
│   │  theme/     (EditorTheme, color,     │      │
│   │             handle, registry)        │      │
│   │  state.rs   (EditMode, Document)     │      │
│   │  utils/     (line helpers)           │      │
│   │  rhai/      (theme engine, plugins)  │      │
│   └──────┬───────────────────────────────┘      │
│          │                                      │
├──────────┼──────────────────────────────────────┤
│          ▼                                      │
│   crates/zoll — парсер разметки                 │
│   ┌──────────────────────────────────────┐      │
│   │  token/   (Tokenizer)                │      │
│   │  parser/  (stack-based AST)          │      │
│   │  ast/     (nodes, markers, style)    │      │
│   │  lib.rs   (parse_document)           │      │
│   └──────────────────────────────────────┘      │
└──────────────────────────────────────────────────┘
```

## Поток данных (цикл кадра)

```
Событие → IcedEditor::update()
  ├─ клавиатура → api::{text,cursor} → dirty = true
  └─ мышь → buffer.hit() → request_redraw()

Кадр → IcedEditor::draw()
  ├─ dirty? → zoll::parse_document()
  │         → layout::compute_line_runs() для каждой строки
  │         → render::shape_document() (cosmic-text Buffer)
  │         → только видимые строки (viewport optimization)
  └─ render: fill_text() для фона, глифов, курсора
```

## Зависимости крейтов

```
main.rs
  └── gui
        ├── api
        │     └── editor
        │           ├── zoll
        │           ├── layout, render, markup, cache
        │           ├── cursor, font, theme, utils, state
        │           └── rhai
        └── editor (через api)
              └── zoll
```

Все зависимости направлены **вниз**: gui → api → editor → zoll. Циклических зависимостей нет.

## Конкурентность

- **Однопоточный** — Iced работает в главном потоке.
- **Синглтоны шрифтов** — `FontSystem` и `SwashCache` обёрнуты в `OnceLock<Mutex<...>>` для безопасного доступа.
- **Нет async** — асинхронных операций нет. Сохранение файла синхронное.
