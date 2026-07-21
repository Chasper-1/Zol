use crate::document::Document;
use crate::editor::cache::DocumentCache;
use crate::editor::render::ShapedDocument;

/// Полное состояние редактора: документ + кеш разметки + shaped-документ.
///
/// Содержит всё, что нужно для рендеринга. API-слой (`api::cursor`, `api::text`)
/// работает с [`Document`] (лёгкая обёртка без cosmic-text).
pub struct EditorWidget {
    pub(crate) doc: Document,
    pub(crate) document_cache: DocumentCache,
    pub(crate) shaped_doc: ShapedDocument,
}

impl EditorWidget {
    pub fn new(text: &str) -> Self {
        let doc = Document::new(text);
        let document_cache = crate::editor::markup::parse_document(&doc.content);
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let empty_buffer = cosmic_text::Buffer::new_empty(metrics);
        let shaped_doc = ShapedDocument::new(empty_buffer);

        Self {
            doc,
            document_cache,
            shaped_doc,
        }
    }

    pub fn content(&self) -> &str {
        &self.doc.content
    }

    #[allow(dead_code)]
    pub fn set_content(&mut self, text: &str) {
        self.doc = Document::new(text);
        self.document_cache = crate::editor::markup::parse_document(&self.doc.content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_with_text() {
        let ew = EditorWidget::new("hello");
        assert_eq!(ew.content(), "hello");
    }

    #[test]
    fn new_empty() {
        let ew = EditorWidget::new("");
        assert_eq!(ew.content(), "");
    }

    #[test]
    fn new_doc_is_dirty() {
        let ew = EditorWidget::new("x");
        assert!(ew.doc.dirty);
    }

    #[test]
    fn content_after_set_content() {
        let mut ew = EditorWidget::new("old");
        ew.set_content("new content");
        assert_eq!(ew.content(), "new content");
    }

    #[test]
    fn set_content_clears_and_sets_dirty() {
        let mut ew = EditorWidget::new("old");
        ew.doc.dirty = false;
        ew.set_content("new");
        assert!(ew.doc.dirty);
    }

    #[test]
    fn set_content_updates_cache() {
        let mut ew = EditorWidget::new("**bold**");
        ew.set_content("plain");
        assert_eq!(ew.content(), "plain");
        // cache should be rebuilt (non-empty, since "plain" is still valid ZML)
        let doc_cache = &ew.document_cache;
        assert_eq!(doc_cache.lines.len(), 1);
    }

    #[test]
    fn multiline_editor_widget() {
        let ew = EditorWidget::new("a\nb\nc");
        assert_eq!(ew.content(), "a\nb\nc");
    }

    #[test]
    fn shaped_doc_created() {
        let ew = EditorWidget::new("hello");
        // shaped_doc exists after construction (not shaped, but buffer exists)
        assert!(ew.shaped_doc.total_height() >= 0.0);
    }
}
