//! Независимое состояние документа: контент + курсор + dirty-флаг.
//!
//! Единственный модуль, который использует `api/`. Никаких зависимостей
//! от cosmic-text, ShapedDocument, DocumentCache или GUI-фреймворков.

use crate::cursor::{self, Cursor};
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
        self.cursor
            .set_raw(&self.incremental.source, &self.incremental.line_starts, raw);
    }

    /// Двигать курсор влево.
    pub fn cursor_move_left(&mut self) {
        self.cursor
            .move_left(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Двигать курсор вправо.
    pub fn cursor_move_right(&mut self) {
        self.cursor
            .move_right(&self.incremental.source, &self.incremental.line_starts);
    }

    /// В начало строки.
    pub fn cursor_move_home(&mut self) {
        self.cursor.move_home(&self.incremental.line_starts);
    }

    /// В конец строки.
    pub fn cursor_move_end(&mut self) {
        self.cursor
            .move_end(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вверх (с сохранением колонки).
    pub fn cursor_move_up(&mut self) {
        self.cursor
            .move_up(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вниз (с сохранением колонки).
    pub fn cursor_move_down(&mut self) {
        self.cursor
            .move_down(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Влево на слово.
    pub fn cursor_move_word_left(&mut self) {
        self.cursor
            .move_word_left(&self.incremental.source, &self.incremental.line_starts);
    }

    /// Вправо на слово.
    pub fn cursor_move_word_right(&mut self) {
        self.cursor
            .move_word_right(&self.incremental.source, &self.incremental.line_starts);
    }

    // ─── Выделение ─────────────────────────────────────

    /// Удалить выделенный текст, если он есть.
    /// Возвращает `true`, если что-то удалено.
    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.cursor.selection_range() {
            self.incremental.edit(start, end, "");
            let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
            self.cursor.set_raw(src, ls, start);
            self.cursor.clear_selection();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Выделить весь текст.
    pub fn select_all(&mut self) {
        if self.content().is_empty() {
            return;
        }
        let len = self.content().len();
        self.cursor.raw = len;
        self.cursor.anchor = Some(0);
        self.cursor.line = self.line_of_byte(len);
        self.dirty = true;
    }

    // ─── Вставка (selection-aware) ─────────────────────

    /// Вставить текст в позицию курсора.
    /// Если есть выделение — заменяет его.
    pub fn insert_at_cursor(&mut self, text: &str) {
        self.delete_selection();
        let raw = self.cursor.raw();
        self.incremental.edit(raw, raw, text);
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, raw + text.len());
        self.dirty = true;
    }

    /// Вставить `\n` в позицию курсора (selection-aware).
    pub fn newline_at_cursor(&mut self) {
        self.delete_selection();
        let raw = self.cursor.raw();
        self.incremental.edit(raw, raw, "\n");
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, raw + 1);
        self.cursor.reset_col_visual();
        self.dirty = true;
    }

    // ─── Удаление (selection-aware) ────────────────────

    /// Удалить grapheme перед курсором (Backspace).
    /// Если есть выделение — удаляет его.
    pub fn delete_before_cursor(&mut self) {
        if self.delete_selection() {
            return;
        }
        let raw = self.cursor.raw();
        if raw == 0 || self.incremental.source.is_empty() {
            return;
        }
        let prev =
            crate::cursor::prev_grapheme_boundary(&self.incremental.source, raw).unwrap_or(0);
        self.incremental.edit(prev, raw, "");
        let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
        self.cursor.set_raw(src, ls, prev);
        self.dirty = true;
    }

    /// Удалить grapheme после курсора (Delete).
    /// Если есть выделение — удаляет его.
    pub fn delete_after_cursor(&mut self) {
        if self.delete_selection() {
            return;
        }
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

    /// Удалить слово перед курсором (Ctrl+Backspace).
    pub fn delete_word_before(&mut self) {
        if self.delete_selection() {
            return;
        }
        let raw = self.cursor.raw();
        let start = cursor::word::prev_word_start(self.content(), raw);
        if start < raw {
            self.incremental.edit(start, raw, "");
            let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
            self.cursor.set_raw(src, ls, start);
            self.dirty = true;
        }
    }

    /// Удалить слово после курсора (Ctrl+Delete).
    pub fn delete_word_after(&mut self) {
        if self.delete_selection() {
            return;
        }
        let raw = self.cursor.raw();
        let end = cursor::word::next_word_start(self.content(), raw);
        if end > raw {
            self.incremental.edit(raw, end, "");
            self.dirty = true;
        }
    }

    /// Удалить всю текущую строку (Ctrl+Shift+Backspace).
    pub fn delete_line(&mut self) {
        let line = self.cursor.line();
        let (start, end) = self
            .line_bounds(line)
            .map(|b| (b.start, b.end))
            .unwrap_or((0, 0));
        // Включаем перенос строки, если не последняя строка
        let end = match self.incremental.line_starts.get(line + 1) {
            Some(&next) => next,
            None => end,
        };
        if end > start {
            self.incremental.edit(start, end, "");
            let (src, ls) = (&self.incremental.source, &self.incremental.line_starts);
            self.cursor.set_raw(src, ls, start);
            self.cursor.clear_selection();
            self.dirty = true;
        }
    }

    /// Удалить от курсора до конца строки (Ctrl+Shift+Delete).
    pub fn delete_to_line_end(&mut self) {
        if self.delete_selection() {
            return;
        }
        let raw = self.cursor.raw();
        let line_end = self.line_end_byte(self.cursor.line());
        if line_end > raw {
            self.incremental.edit(raw, line_end, "");
            self.dirty = true;
        }
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
        self.line_bounds(line)
            .map(|b| unsafe { self.incremental.source.get_unchecked(b.start..b.end) })
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

    /// Конечный байт строки (позиция после последнего символа, без \n).
    pub fn line_end_byte(&self, line: usize) -> usize {
        self.line_bounds(line).map(|b| b.end).unwrap_or(0)
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

    // ─── Выделение ─────────────────────────────────────

    #[test]
    fn delete_selection_removes_range() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 11;
        doc.cursor.anchor = Some(6);
        assert!(doc.delete_selection());
        assert_eq!(doc.content(), "hello ");
        assert_eq!(doc.cursor.raw(), 6);
        assert!(!doc.cursor.has_selection());
    }

    #[test]
    fn delete_selection_no_selection() {
        let mut doc = Document::new("hello");
        assert!(!doc.delete_selection());
        assert_eq!(doc.content(), "hello");
    }

    #[test]
    fn select_all_selects_entire_content() {
        let mut doc = Document::new("hello world");
        doc.select_all();
        assert_eq!(doc.cursor.selection_range(), Some((0, 11)));
    }

    #[test]
    fn select_all_empty_doc() {
        let mut doc = Document::new("");
        doc.select_all();
        assert!(doc.cursor.selection_range().is_none());
    }

    #[test]
    fn insert_at_cursor_replaces_selection() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 11;
        doc.cursor.anchor = Some(6);
        doc.insert_at_cursor("there");
        assert_eq!(doc.content(), "hello there");
    }

    #[test]
    fn delete_before_with_selection_deletes_selection() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 11;
        doc.cursor.anchor = Some(6);
        doc.delete_before_cursor();
        assert_eq!(doc.content(), "hello ");
    }

    #[test]
    fn delete_after_with_selection_deletes_selection() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 5;
        doc.cursor.anchor = Some(0);
        doc.delete_after_cursor();
        assert_eq!(doc.content(), " world");
    }

    #[test]
    fn newline_replaces_selection() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 11;
        doc.cursor.anchor = Some(6);
        doc.newline_at_cursor();
        // "world" (6..11) deleted → "hello ", then '\n' inserted → "hello \n"
        assert_eq!(doc.content(), "hello \n");
    }

    // ─── Удаление слов ────────────────────────────

    #[test]
    fn delete_word_before_mid_second_word() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 7; // 'o' in 'world'
        doc.delete_word_before();
        assert_eq!(doc.content(), "hello orld"); // 'w' deleted (word start = 6)
        assert_eq!(doc.cursor.raw(), 6);
    }

    #[test]
    fn delete_word_before_from_end() {
        let mut doc = Document::new("hello world foo");
        doc.cursor.raw = 15; // past end
        doc.delete_word_before();
        assert_eq!(doc.content(), "hello world "); // 'foo' deleted
        assert_eq!(doc.cursor.raw(), 12);
    }

    #[test]
    fn delete_word_after_from_start() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 0;
        doc.delete_word_after();
        assert_eq!(doc.content(), "world"); // 'hello ' deleted (0..6)
    }

    // ─── Удаление строк ───────────────────────────

    #[test]
    fn delete_line_middle_line() {
        let mut doc = Document::new("a\nb\nc");
        doc.set_cursor_raw(2); // 'b' on line 1
        doc.delete_line();
        assert_eq!(doc.content(), "a\nc");
    }

    #[test]
    fn delete_line_last_line() {
        let mut doc = Document::new("a\nb\nc");
        doc.set_cursor_raw(4); // 'c' on line 2
        doc.delete_line();
        assert_eq!(doc.content(), "a\nb\n");
    }

    #[test]
    fn delete_to_line_end() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 5; // after 'hello'
        doc.delete_to_line_end();
        assert_eq!(doc.content(), "hello");
    }

    #[test]
    fn delete_to_line_end_with_selection_first() {
        let mut doc = Document::new("hello world");
        doc.cursor.raw = 11;
        doc.cursor.anchor = Some(6);
        doc.delete_to_line_end();
        assert_eq!(doc.content(), "hello ");
    }
}
