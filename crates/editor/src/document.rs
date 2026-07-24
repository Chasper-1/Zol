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
        self.cursor.set_raw(&self.incremental.source, &self.incremental.line_starts, raw);
    }

    /// Двигать курсор влево.
    pub fn cursor_move_left(&mut self) {
        self.cursor.move_left(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Двигать курсор вправо.
    pub fn cursor_move_right(&mut self) {
        self.cursor.move_right(&self.incremental.source, &self.incremental.line_starts);
    }

    /// В начало строки.
    pub fn cursor_move_home(&mut self) {
        self.cursor.move_home(&self.incremental.source, &self.incremental.line_starts);
    }

    /// В конец строки.
    pub fn cursor_move_end(&mut self) {
        self.cursor.move_end(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вверх (с сохранением колонки).
    pub fn cursor_move_up(&mut self) {
        self.cursor.move_up(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вниз (с сохранением колонки).
    pub fn cursor_move_down(&mut self) {
        self.cursor.move_down(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Влево на слово.
    pub fn cursor_move_word_left(&mut self) {
        self.cursor.move_word_left(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вправо на слово.
    pub fn cursor_move_word_right(&mut self) {
        self.cursor.move_word_right(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вставить текст в позицию курсора.
    pub fn insert_at_cursor(&mut self, text: &str) {
        let raw = self.cursor.raw();
        self.incremental.edit(raw, raw, text);
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, raw + text.len());
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
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, prev);
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
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, raw);
        self.dirty = true;
    }

    /// Вставить `\n` в позицию курсора.
    pub fn newline_at_cursor(&mut self) {
        let raw = self.cursor.raw();
        self.incremental.edit(raw, raw, "\n");
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, raw + 1);
        self.cursor.reset_col_visual();
        self.dirty = true;
    }

    // ─── O(1) line helpers via IncrementalDoc.line_starts ────────

    /// Границы строки (start..end) по индексу.
    pub fn line_bounds(&self, line: usize) -> Option<crate::utils::LineBounds> {
        let starts = &self.incremental.line_starts;
        let start = *starts.get(line)?;
        let end = starts
            .get(line + 1)
            .map(|&next| next.saturating_sub(1))
            .unwrap_or(self.incremental.source.len());
        Some(crate::utils::LineBounds { start, end })
    }

    /// Текст строки по индексу.
    pub fn line_text(&self, line: usize) -> Option<&str> {
        self.line_bounds(line).map(|b| {
            unsafe { self.incremental.source.get_unchecked(b.start..b.end) }
        })
    }

    /// Номер строки, содержащей байтовую позицию (O(log n) бинарный поиск).
    pub fn line_of_byte(&self, byte: usize) -> usize {
        let starts = &self.incremental.line_starts;
        if self.incremental.source.is_empty() || starts.is_empty() || byte == 0 {
            return 0;
        }
        let byte_pos = byte.min(self.incremental.source.len());
        match starts.binary_search(&byte_pos) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        }
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
