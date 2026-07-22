//! Zoll markup language — чистый парсер разметки.
//!
//! Зависимостей нет, только `std`.
//!
//! - [`ast`] — AST (MarkupDoc, MarkupNode, MarkupStyle, MarkerDef)
//! - [`token`] — токенизатор (Token, tokenize)
//! - [`parser`] — стековый парсер (Token → MarkupDoc)

pub mod ast;
pub mod parser;
pub mod token;
