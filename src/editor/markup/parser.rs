use crate::editor::cache::DocumentCache;

pub fn parse_document(text: &str) -> DocumentCache {
    crate::mdplus::parse_document(text)
}
