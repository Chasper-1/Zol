//! Независимое состояние документа: контент + курсор + dirty-флаг.
//!
//! Единственный модуль, который использует `api/`. Никаких зависимостей
//! от cosmic-text, ShapedDocument, DocumentCache или GUI-фреймворков.

use crate::editor::cursor::Cursor;

/// Состояние редактируемого документа.
///
/// Содержит только то, что нужно API-операциям (move, insert, delete).
/// Всё, что связано с рендерингом (ShapedDocument, DocumentCache) —
/// в `editor::editor_widget::EditorWidget`.
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
