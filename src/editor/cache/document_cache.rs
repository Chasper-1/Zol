use super::markup_cache::MarkupCache;

#[derive(Default, Clone, Debug)]
pub struct DocumentCache {
    pub lines: Vec<MarkupCache>,
}
