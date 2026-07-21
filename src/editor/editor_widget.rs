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
