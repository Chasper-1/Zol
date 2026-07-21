//! Независимое состояние документа: контент + курсор + dirty-флаг.
//!
//! Единственный модуль, который использует `api/`. Никаких зависимостей
//! от cosmic-text, ShapedDocument, DocumentCache или GUI-фреймворков.

use crate::editor::cursor::Cursor;

/// Состояние редактируемого документа.
///
/// Содержит только то, что нужно API-операциям (move, insert, delete).
/// Всё, что связано с рендерингом (ShapedDocument, DocumentCache) —
/// в `gui::iced_editor::EditorInner`.
pub struct Document {
    pub content: String,
    pub cursor: Cursor,
    pub dirty: bool,
}

impl Document {
    pub fn new(text: &str) -> Self {
        Self {
            content: text.to_string(),
            cursor: Cursor::new(),
            dirty: true,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_document_with_content() {
        let doc = Document::new("hello world");
        assert_eq!(doc.content, "hello world");
    }

    #[test]
    fn new_empty_content() {
        let doc = Document::new("");
        assert_eq!(doc.content, "");
    }

    #[test]
    fn new_is_dirty() {
        let doc = Document::new("x");
        assert!(doc.dirty);
    }

    #[test]
    fn new_cursor_at_start() {
        let doc = Document::new("abc");
        assert_eq!(doc.cursor.raw(), 0);
        assert_eq!(doc.cursor.line(), 0);
    }

    #[test]
    fn content_returns_same_as_field() {
        let doc = Document::new("test content");
        assert_eq!(doc.content(), doc.content.as_str());
    }

    #[test]
    fn multiline_content() {
        let doc = Document::new("line1\nline2\nline3");
        assert_eq!(doc.content.lines().count(), 3);
    }

    #[test]
    fn content_with_unicode() {
        let text = "привет мир 👋";
        let doc = Document::new(text);
        assert_eq!(doc.content(), text);
    }
}
