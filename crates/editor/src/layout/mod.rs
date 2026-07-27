//! Независимая раскладка текста (0 зависимостей от GUI).
//!
//! - [`types`] — `TextRun`, `LineLayout` — чистые типы
//! - [`compute`] — разбор строки в `TextRun` по сегментам разметки

pub mod compute;
pub mod types;

pub use compute::cursor_line_bounds;
pub use types::TextRun;
