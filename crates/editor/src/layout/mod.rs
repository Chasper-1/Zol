//! Независимая раскладка текста (0 зависимостей от GUI).
//!
//! - [`types`] — `TextRun`, `LineLayout` — чистые типы
//! - [`compute`] — разбор строки в `TextRun` по сегментам разметки
//! - [`reveal`] — авто-раскрытие маркеров по положению курсора

pub mod compute;
pub mod reveal;
pub mod types;

pub use compute::cursor_line_bounds;
pub use types::TextRun;
