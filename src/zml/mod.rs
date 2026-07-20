pub mod ast;
pub mod token;
pub mod parser;
pub mod segmenter;

use crate::editor::cache::DocumentCache;

/// Парсит zml-текст в DocumentCache для редактора.
pub fn parse_document(text: &str) -> DocumentCache {
    let tokens = token::tokenize(text);
    let ast = parser::parse(&tokens);
    segmenter::to_document_cache(&ast)
}
