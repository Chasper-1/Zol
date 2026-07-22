//! Стековый парсер zoll.
//!
//! Преобразует поток токенов в AST (MarkupDoc).
//! Использует стек вместо рекурсии — нет переполнения стека на глубокой вложенности.

mod marker_text;
mod parse;
mod tests;

pub use parse::parse;
