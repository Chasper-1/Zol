//! Токенизация zoll-текста.
//!
//! Преобразует сырой текст в последовательность [`Token`] за один проход.

mod find_deep_close;
mod helpers;
mod push_text;
mod tests;
mod tokenize;
mod types;

pub use tokenize::tokenize;
pub use types::{SpannedToken, Token};
