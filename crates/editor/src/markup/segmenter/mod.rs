//! Преобразование AST zoll в DocumentCache для редактора.
//!
//! Зависит от `zoll-core` (AST) и `editor::cache` / `editor::markup::segment`.

mod build;
mod helpers;
mod incremental;
#[cfg(test)] mod tests;
mod to_doc;

pub use incremental::incremental_to_cache;
pub use to_doc::to_document_cache;
