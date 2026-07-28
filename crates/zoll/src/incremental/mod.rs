//! Инкрементальный парсер zoll.
//!
//! Единственная структура — `IncrementalDoc`, обёртка над `ParsedDoc`.
//! При редактировании перепарсивает только изменённые строки.

mod doc;

pub use doc::{IncrementalDoc, build_line_starts};
