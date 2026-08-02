use super::markup_cache::MarkupCache;

#[derive(Clone, Debug)]
pub struct DocumentCache {
    pub lines: Vec<MarkupCache>,
    // Для каждой строки — ID блока (если строка внутри блока), иначе None.
    pub block_of_line: Vec<Option<usize>>,
}

impl Default for DocumentCache {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            block_of_line: Vec::new(),
        }
    }
}
