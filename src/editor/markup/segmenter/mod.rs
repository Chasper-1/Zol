//! Преобразование AST zoll в DocumentCache для редактора.
//!
//! Зависит от `zoll-core` (AST) и `editor::cache` / `editor::markup::segment`.

mod build;
mod helpers;
#[cfg(test)] mod tests;
mod to_doc;

pub use to_doc::to_document_cache;
