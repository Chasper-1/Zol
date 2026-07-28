//! Zoll markup language — line-level парсер разметки.
//!
//! - [`model`] — модель разобранного документа (ParsedDoc, ParsedLine, BlockKind, Segment, MarkStyle)
//! - [`parser`] — парсер одной строки (parse_line → ParsedLine)
//! - [`incremental`] — инкрементальный документ (IncrementalDoc)
//! - [`viewport`] — видимый диапазон строк (Viewport)

pub mod incremental;
pub mod model;
pub mod parser;
pub mod viewport;

// Реэкспорт основного для удобства пользователей.
pub use model::{BlockKind, MarkStyle, ParsedDoc, ParsedLine, Segment};
pub use viewport::Viewport;
