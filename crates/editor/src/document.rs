//! Независимое состояние документа: контент + курсор + dirty-флаг.
//!
//! Единственный модуль, который использует `api/`. Никаких зависимостей
//! от cosmic-text, ShapedDocument, DocumentCache или GUI-фреймворков.

use crate::cursor::Cursor;
use zoll::incremental::IncrementalDoc;

/// Состояние редактируемого документа.
///
/// Содержит только то, что нужно API-операциям (move, insert, delete).
/// Всё, что связано с рендерингом (ShapedDocument, DocumentCache) —
/// в `gui::iced_editor::EditorInner`.
pub struct Document {
    /// Инкрементальный парсер: source + line_starts + line_asts + merged_ast.
    pub incremental: IncrementalDoc,
    /// Позиция курсора (байт, строка, визуальная колонка).
    pub cursor: Cursor,
    /// Флаг: нужно перестроить ShapedDocument.
    pub dirty: bool,
}

impl Document {
    /// Создать новый документ из текста.
    pub fn new(text: &str) -> Self {
        Self {
            incremental: IncrementalDoc::new(text),
            cursor: Cursor::new(),
            dirty: true,
        }
    }

    /// Получить сырой текст документа.
    pub fn content(&self) -> &str {
        &self.incremental.source
    }

    // ─── Обёртки для cursor + content (borrow-checker helper) ────────

    /// Установить курсор на байт (с проверкой границ).
    pub fn set_cursor_raw(&mut self, raw: usize) {
        self.cursor.set_raw(&self.incremental.source, raw);
    }

    /// Двигать курсор влево.
    pub fn cursor_move_left(&mut self) {
        self.cursor.move_left(&self.incremental.source);
    }

    /// Двигать курсор вправо.
    pub fn cursor_move_right(&mut self) {
        self.cursor.move_right(&self.incremental.source);
    }

    /// В начало строки.
    pub fn cursor_move_home(&mut self) {
        self.cursor.move_home(&self.incremental.source);
    }

    /// В конец строки.
    pub fn cursor_move_end(&mut self) {
        self.cursor.move_end(&self.incremental.source);
    }

    /// Вверх (с сохранением колонки).
    pub fn cursor_move_up(&mut self) {
        self.cursor.move_up(&self.incremental.source);
    }

    /// Вниз (с сохранением колонки).
    pub fn cursor_move_down(&mut self) {
        self.cursor.move_down(&self.incremental.source);
    }

    /// Влево на слово.
    pub fn cursor_move_word_left(&mut self) {
        self.cursor.move_word_left(&self.incremental.source);
    }

    /// Вправо на слово.
    pub fn cursor_move_word_right(&mut self) {
        self.cursor.move_word_right(&self.incremental.source);
    }

    /// Вставить текст в позицию курсора.
    pub fn insert_at_cursor(&mut self, text: &str) {
        let raw = self.cursor.raw();
        self.incremental.edit(raw, raw, text);
        self.cursor.set_raw(&self.incremental.source, raw + text.len());
        self.dirty = true;
    }

    /// Удалить grapheme перед курсором.
    pub fn delete_before_cursor(&mut self) {
        let raw = self.cursor.raw();
        if raw == 0 || self.incremental.source.is_empty() {
            return;
        }
        let prev = crate::cursor::prev_grapheme_boundary(&self.incremental.source, raw).unwrap_or(0);
        self.incremental.edit(prev, raw, "");
        self.cursor.set_raw(&self.incremental.source, prev);
        self.dirty = true;
    }

    /// Удалить grapheme после курсора.
    pub fn delete_after_cursor(&mut self) {
        let raw = self.cursor.raw();
        if raw >= self.incremental.source.len() || self.incremental.source.is_empty() {
            return;
        }
        let next = crate::cursor::next_grapheme_boundary(&self.incremental.source, raw)
            .unwrap_or(self.incremental.source.len());
        self.incremental.edit(raw, next, "");
        self.cursor.set_raw(&self.incremental.source, raw);
        self.dirty = true;
    }

    /// Вставить `\n` в позицию курсора.
    pub fn newline_at_cursor(&mut self) {
        let raw = self.cursor.raw();
        self.incremental.edit(raw, raw, "\n");
        self.cursor.set_raw(&self.incremental.source, raw + 1);
        self.cursor.reset_col_visual();
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_document_with_content() {
        let doc = Document::new("hello world");
        assert_eq!(doc.content(), "hello world");
    }

    #[test]
    fn new_empty_content() {
        let doc = Document::new("");
        assert_eq!(doc.content(), "");
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
        assert_eq!(doc.content(), doc.incremental.source.as_str());
    }

    #[test]
    fn multiline_content() {
        let doc = Document::new("line1\nline2\nline3");
        assert_eq!(doc.content().lines().count(), 3);
    }

    #[test]
    fn content_with_unicode() {
        let text = "привет мир 👋";
        let doc = Document::new(text);
        assert_eq!(doc.content(), text);
    }
}
