use crate::editor::cursor::{self, Cursor};
use crate::editor::editor_widget::EditorWidget;
use crate::editor::utils::line_utils;

pub fn insert_at_cursor(widget: &mut EditorWidget, text: &str) {
    let raw = widget.cursor.raw();
    widget.content.insert_str(raw, text);
    widget.cursor.set_raw(&widget.content, raw + text.len());
    widget.dirty = true;
}

/// Удалить **grapheme-кластер** перед курсором.
pub fn delete_before_cursor(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw();
    if raw == 0 || widget.content.is_empty() {
        return;
    }
    let prev = cursor::prev_grapheme_boundary(&widget.content, raw).unwrap_or(0);
    widget.content.drain(prev..raw);
    widget.cursor.set_raw(&widget.content, prev);
    widget.dirty = true;
}

/// Удалить **grapheme-кластер** после курсора.
pub fn delete_after_cursor(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw();
    if raw >= widget.content.len() || widget.content.is_empty() {
        return;
    }
    let next = cursor::next_grapheme_boundary(&widget.content, raw).unwrap_or(widget.content.len());
    widget.content.drain(raw..next);
    widget.cursor.set_raw(&widget.content, raw);
    widget.dirty = true;
}

pub fn newline(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw();
    widget.content.insert(raw, '\n');
    widget.cursor.set_raw(&widget.content, raw + 1);
    widget.cursor.reset_col_visual();
    widget.dirty = true;
}

#[allow(dead_code)]
pub fn get_text(widget: &EditorWidget) -> &str {
    &widget.content
}

#[allow(dead_code)]
pub fn get_line(widget: &EditorWidget, idx: usize) -> Option<&str> {
    line_utils::line_text(&widget.content, idx)
}

#[allow(dead_code)]
pub fn get_line_count(widget: &EditorWidget) -> usize {
    line_utils::count_lines(&widget.content)
}

#[allow(dead_code)]
pub fn text_len(widget: &EditorWidget) -> usize {
    widget.content.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::editor_widget::EditorWidget;

    fn make_widget(text: &str) -> EditorWidget {
        EditorWidget::new(text)
    }

    #[test]
    fn insert_at_cursor_adds_text() {
        let mut w = make_widget("hello");
        w.cursor.set_raw(&w.content, 5);
        insert_at_cursor(&mut w, " world");
        assert_eq!(w.content, "hello world");
        assert_eq!(w.cursor.raw(), "hello world".len());
    }

    #[test]
    fn insert_at_cursor_mid_text() {
        let mut w = make_widget("helo");
        w.cursor.set_raw(&w.content, 3);
        insert_at_cursor(&mut w, "l");
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn delete_before_cursor_removes_char() {
        let mut w = make_widget("hello");
        w.cursor.set_raw(&w.content, 5);
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "hell");
        assert_eq!(w.cursor.raw(), 4);
    }

    #[test]
    fn delete_before_cursor_at_start() {
        let mut w = make_widget("hello");
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn delete_before_cursor_empty() {
        let mut w = make_widget("");
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "");
    }

    #[test]
    fn delete_after_cursor_removes_char() {
        let mut w = make_widget("hello");
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "ello");
    }

    #[test]
    fn delete_after_cursor_at_end() {
        let mut w = make_widget("hello");
        w.cursor.set_raw(&w.content, 5);
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn newline_inserts_newline() {
        let mut w = make_widget("ab");
        w.cursor.set_raw(&w.content, 1);
        newline(&mut w);
        assert_eq!(w.content, "a\nb");
        assert_eq!(w.cursor.raw(), 2);
    }

    #[test]
    fn get_text_returns_content() {
        let w = make_widget("hello");
        assert_eq!(get_text(&w), "hello");
    }

    #[test]
    fn get_line_count_works() {
        let w = make_widget("a\nb\nc");
        assert_eq!(get_line_count(&w), 3);
    }

    #[test]
    fn get_line_works() {
        let w = make_widget("first\nsecond\nthird");
        assert_eq!(get_line(&w, 0), Some("first"));
        assert_eq!(get_line(&w, 1), Some("second"));
        assert_eq!(get_line(&w, 2), Some("third"));
        assert_eq!(get_line(&w, 3), None);
    }

    #[test]
    fn text_len_works() {
        let w = make_widget("hello");
        assert_eq!(text_len(&w), 5);
    }

    #[test]
    fn unicode_insert() {
        let mut w = make_widget("Приве");
        w.cursor.set_raw(&w.content, 10);
        insert_at_cursor(&mut w, "т");
        assert_eq!(w.content, "Привет");
        assert_eq!(w.cursor.raw(), 12);
    }

    #[test]
    fn unicode_delete_before() {
        let mut w = make_widget("Привет");
        w.cursor.set_raw(&w.content, 12);
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "Приве");
        assert_eq!(w.cursor.raw(), 10);
    }

    #[test]
    fn grapheme_delete_before() {
        let mut w = make_widget("e\u{0301}x");
        w.cursor.set_raw(&w.content, 4);
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "e\u{0301}");
    }

    #[test]
    fn grapheme_delete_after() {
        let mut w = make_widget("e\u{0301}x");
        w.cursor.set_raw(&w.content, 0);
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "x");
    }
}
