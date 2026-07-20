# Архитектура

## Диаграмма слоёв

```
┌─────────────────────────────────────────────────────────┐
│                      Бинарник Zol                       │
│  main.rs (флаг --iced выбирает egui или Iced)           │
├─────────────────────────────────────────────────────────┤
│                    Слой GUI                              │
│  gui/ (egui::App / Iced::Application)                   │
│  ┌─────────────────┐  ┌──────────────────────────────┐  │
│  │  egui backend   │  │  Iced backend (эксперимент)  │  │
│  │  app.rs, run.rs │  │  app_iced.rs, iced_editor.rs │  │
│  └────────┬────────┘  └──────────────┬───────────────┘  │
│           │                          │                   │
├───────────┼──────────────────────────┼───────────────────┤
│           ▼                          ▼                   │
│  ┌──────────────────────────────────────────────┐       │
│  │          Ядро редактора                       │       │
│  │  editor/ (не зависит от GUI)                 │       │
│  │  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │       │
│  │  │ Cursor   │  │ Layout   │  │ Render       │  │       │
│  │  │ cursor.rs│  │ compute/ │  │ shape/paint  │  │       │
│  │  └──────────┘  └──────────┘  └──────┬───────┘  │       │
│  │  ┌──────────┐  ┌──────────┐         │          │       │
│  │  │ Markup   │  │ Cache    │         │          │       │
│  │  │ mdplus   │→│ Document │         │          │       │
│  │  └──────────┘  └──────────┘         │          │       │
│  │  ┌──────────────────────────┐       │          │       │
│  │  │ Font (font.rs)           │←──────│          │       │
│  │  └──────────────────────────┘                │       │
│  └──────────────────────────────────────────────┘       │
├─────────────────────────────────────────────────────────┤
│                    Слой API                               │
│  api/ (публичный интерфейс для Rhai-плагинов)            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ cursor   │  │ text     │  │ editor   │               │
│  └──────────┘  └──────────┘  └──────────┘               │
├─────────────────────────────────────────────────────────┤
│                    Слой Rhai                              │
│  rhai/ (движок темы, плагины)                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ engine/  │  │ plugins/ │  │ system/  │               │
│  │          │  │          │  │ languages│               │
│  └──────────┘  └──────────┘  └──────────┘               │
└─────────────────────────────────────────────────────────┘
```

## Поток данных (цикл кадра)

### egui backend

```
начало кадра
  │
  ├─ EditorWidget::ui()
  │   ├─ handle_input() → api::{text,cursor}::* → dirty=true
  │   ├─ dirty || input?
  │   │   ├─ mdplus::parse_document() → DocumentCache
  │   │   └─ render::build() → ShapedDocument (cosmic-text Buffer)
  │   └─ render::paint() → egui отрисовка
  │
  └─ конец кадра
```

### Iced backend

```
Событие → IcedEditor::update()
  ├─ клавиатура → изменить content/cursor, dirty.set(true)
  └─ мышь → buffer.hit(), установить курсор, request_redraw()

Кадр → IcedEditor::draw()
  ├─ dirty? → render::build() с высотой вьюпорта
  └─ fill_quad() для каждого глифа + курсор
```

## Зависимости модулей

```
main.rs
  ├── gui::run         (точка входа egui)
  │     └── gui::app::ZolApp
  │           └── editor::editor_widget::EditorWidget
  │                 ├── editor::cursor
  │                 ├── editor::input
  │                 ├── editor::render (build + paint)
  │                 ├── editor::layout::compute
  │                 ├── editor::font
  │                 ├── editor::markup
  │                 ├── editor::cache
  │                 └── editor::state
  ├── gui::app_iced    (точка входа Iced)
  │     └── gui::iced_editor::IcedEditor
  │           └── editor::render (build)
  │               └── editor::font
  ├── api              (публичное API)
  │     ├── api::cursor
  │     ├── api::text
  │     └── api::editor
  └── mdplus           (парсер разметки)
        ├── mdplus::token
        ├── mdplus::parser
        ├── mdplus::ast
        └── mdplus::segmenter
```

## Модель конкурентности

- **Однопоточный** — и egui, и Iced работают в главном потоке.
- **Синглтоны шрифтов** — `FontSystem` и `SwashCache` обёрнуты в `OnceLock<Mutex<...>>` для безопасного доступа, хотя пока используется только главный поток.
- **Нет async** — асинхронных операций в редакторе нет. Файловый ввод/вывод (сохранение) синхронный.
