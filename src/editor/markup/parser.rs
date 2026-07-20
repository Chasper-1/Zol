use crate::editor::cache::DocumentCache;

pub fn parse_document(text: &str) -> DocumentCache {
    crate::zml::parse_document(text)
}
