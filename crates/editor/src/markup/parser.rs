use crate::cache::DocumentCache;
use crate::markup::segmenter::to_document_cache;

/// Парсит zoll-текст в DocumentCache для редактора.
pub fn parse_document(text: &str) -> DocumentCache {
    let tokens = zoll::token::tokenize(text);
    let ast = zoll::parser::parse(&tokens);
    to_document_cache(&ast)
}
