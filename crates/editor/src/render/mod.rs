//! Рендеринг: шейпинг текста через cosmic-text.

mod build;
mod shape;
mod shaped_doc;

pub use build::build;
pub use shaped_doc::ShapedDocument;

#[cfg(test)]
mod tests;
