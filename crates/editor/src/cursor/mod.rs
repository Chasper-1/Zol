//! Позиция курсора, выделение, мигание.
//!
//! Все move/delete-операции централизованы в `input::default::InputModel`.

pub mod grapheme;
pub mod marker_skip;
pub mod types;

pub use grapheme::{next_grapheme_boundary, prev_grapheme_boundary};
pub use types::Cursor;

#[cfg(test)]
mod tests;
