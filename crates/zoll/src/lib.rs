//! Zoll markup language — чистый парсер разметки.
//!
//! Зависимостей нет, только `std`.
//!
//! - [`ast`] — AST (MarkupDoc, MarkupNode, MarkupStyle, MarkerDef)
//! - [`token`] — токенизатор (Token, tokenize, SpannedToken)
//! - [`parser`] — стековый парсер (Token → MarkupDoc)
//! - [`incremental`] — инкрементальный парсер (IncrementalDoc)

pub mod ast;
pub mod incremental;
pub mod parser;
pub mod token;
