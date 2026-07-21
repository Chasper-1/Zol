use crate::editor::cache::DocumentCache;

pub fn parse_document(text: &str) -> DocumentCache {
    crate::zoll::parse_document(text)
}
