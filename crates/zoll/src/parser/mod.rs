//! Строчный парсер zoll.
//!
//! Единственная функция: `parse_line(&str) → ParsedLine`.
//! Никакого merge, никаких деревьев — парсер сразу пишет в кеш-модель.

mod line;

pub use line::parse_line;
