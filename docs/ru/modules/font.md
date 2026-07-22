# Модуль шрифтов

`crates/editor/src/font/` — глобальная система шрифтов Zol.

## Архитектура синглтона

```rust
static GLOBAL: OnceLock<Mutex<FontGlobal>> = OnceLock::new();

struct FontGlobal {
    font_system: cosmic_text::FontSystem,
    swash_cache: cosmic_text::SwashCache,
}
```

- **`OnceLock`** — инициализируется ровно один раз, thread-safe (хотя Zol однопоточный).
- **`Mutex`** — обеспечивает безопасный доступ из любого потока, на будущее (например, фоновый шейпинг).

## API

```rust
// Инициализация (идемпотентна — повторные вызовы игнорируются)
pub fn init();

// Доступ к FontSystem для операций шейпинга
pub fn with_font_system<F, T>(f: F) -> T
where F: FnOnce(&mut cosmic_text::FontSystem) -> T;

// Доступ к SwashCache для растрирования глифов
pub fn with_swash_cache<F, T>(f: F) -> T
where F: FnOnce(&mut cosmic_text::SwashCache) -> T;

// Доступ к FontSystem и SwashCache одновременно (избегает взаимной блокировки)
pub fn with_font_and_cache<F, T>(f: F) -> T
where F: FnOnce(&mut cosmic_text::FontSystem, &mut cosmic_text::SwashCache) -> T;

// Список всех доступных семейств шрифтов
pub fn list_families() -> Vec<String>;

// Пересканировать системные шрифты (например, после установки нового шрифта)
pub fn reload_system_fonts();
```

## Инициализация

`init()` вызывается автоматически из `render::build()` при первом вызове:

1. Создаёт `fontdb::Database`
2. Вызывает `db.load_system_fonts()` — загружает все шрифты из ОС
3. Создаёт `cosmic_text::FontSystem` с локалью "en" и заполненной БД
4. Создаёт пустой `cosmic_text::SwashCache`
5. Сохраняет всё в глобальный `OnceLock`

## Использование в рендере

```
render::build()
    │
    ├─ font::init()  (только первый вызов)
    │
    └─ font::with_font_system(|fs| {
           shape_document(&runs, fs, base_size, font_family, viewport_height)
       })
```

`with_font_system` блокирует глобальный мьютекс и предоставляет `&mut FontSystem` замыканию. Мьютекс удерживается только на время выполнения замыкания.

## Зависимости

- `cosmic-text` v0.19 — движок шейпинга
- `fontdb` (через реэкспорт cosmic-text) — база шрифтов и их обнаружение
- Других крейтов для шрифтов нет (нет `fontconfig`, `freetype` или системных библиотек сверх тех, что использует cosmic-text)

## Планы

- Встроенные шрифты: загрузка `.ttf` из каталога ресурсов
- Настройка fallback-шрифтов через тему
- Выбор по начертанию (weight, style) вместо только семейства
