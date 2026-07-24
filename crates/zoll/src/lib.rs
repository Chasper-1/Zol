//! Zoll markup language — чистый парсер разметки.
//!
//! Зависимостей нет, только `std`.
//!
//! - [`ast`] — AST (MarkupDoc, MarkupNode, MarkupStyle, LineAST)
//! - [`parser`] — строчный парсер (parse_line, merge)
//! - [`incremental`] — инкрементальный парсер (IncrementalDoc)

pub mod ast;
pub mod incremental;
pub mod parser;
