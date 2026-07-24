use crate::cache::DocumentCache;
use crate::markup::segmenter::to_document_cache;

/// Парсит zoll-текст в DocumentCache для редактора.
pub fn parse_document(text: &str) -> DocumentCache {
    let ast = zoll::parser::parse_full(text);
    to_document_cache(&ast)
}
