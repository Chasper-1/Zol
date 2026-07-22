//! Позиция курсора, навигация, мигание.

pub mod grapheme;
pub mod movement;
pub mod types;
pub mod word;

pub use grapheme::{next_grapheme_boundary, prev_grapheme_boundary};
pub use types::Cursor;

#[cfg(test)]
mod tests;
