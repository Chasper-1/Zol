use crate::document::Document;
use crate::editor::cursor;
use crate::editor::utils::line_utils;

pub fn insert_at_cursor(doc: &mut Document, text: &str) {
    let raw = doc.cursor.raw();
    doc.content.insert_str(raw, text);
    doc.cursor.set_raw(&doc.content, raw + text.len());
    doc.dirty = true;
}

/// Удалить **grapheme-кластер** перед курсором.
pub fn delete_before_cursor(doc: &mut Document) {
    let raw = doc.cursor.raw();
    if raw == 0 || doc.content.is_empty() {
        return;
    }
    let prev = cursor::prev_grapheme_boundary(&doc.content, raw).unwrap_or(0);
    doc.content.drain(prev..raw);
    doc.cursor.set_raw(&doc.content, prev);
    doc.dirty = true;
}

/// Удалить **grapheme-кластер** после курсора.
pub fn delete_after_cursor(doc: &mut Document) {
    let raw = doc.cursor.raw();
    if raw >= doc.content.len() || doc.content.is_empty() {
        return;
    }
    let next = cursor::next_grapheme_boundary(&doc.content, raw).unwrap_or(doc.content.len());
    doc.content.drain(raw..next);
    doc.cursor.set_raw(&doc.content, raw);
    doc.dirty = true;
}

pub fn newline(doc: &mut Document) {
    let raw = doc.cursor.raw();
    doc.content.insert(raw, '\n');
    doc.cursor.set_raw(&doc.content, raw + 1);
    doc.cursor.reset_col_visual();
    doc.dirty = true;
}

#[allow(dead_code)]
pub fn get_text(doc: &Document) -> &str {
    &doc.content
}

#[allow(dead_code)]
pub fn get_line(doc: &Document, idx: usize) -> Option<&str> {
    line_utils::line_text(&doc.content, idx)
}

#[allow(dead_code)]
pub fn get_line_count(doc: &Document) -> usize {
    line_utils::count_lines(&doc.content)
}

#[allow(dead_code)]
pub fn text_len(doc: &Document) -> usize {
    doc.content.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    fn make_doc(text: &str) -> Document {
        Document::new(text)
    }

    #[test]
    fn insert_at_cursor_adds_text() {
        let mut w = make_doc("hello");
        w.cursor.set_raw(&w.content, 5);
        insert_at_cursor(&mut w, " world");
        assert_eq!(w.content, "hello world");
        assert_eq!(w.cursor.raw(), "hello world".len());
    }

    #[test]
    fn insert_at_cursor_mid_text() {
        let mut w = make_doc("helo");
        w.cursor.set_raw(&w.content, 3);
        insert_at_cursor(&mut w, "l");
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn delete_before_cursor_removes_char() {
        let mut w = make_doc("hello");
        w.cursor.set_raw(&w.content, 5);
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "hell");
        assert_eq!(w.cursor.raw(), 4);
    }

    #[test]
    fn delete_before_cursor_at_start() {
        let mut w = make_doc("hello");
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn delete_before_cursor_empty() {
        let mut w = make_doc("");
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "");
    }

    #[test]
    fn delete_after_cursor_removes_char() {
        let mut w = make_doc("hello");
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "ello");
    }

    #[test]
    fn delete_after_cursor_at_end() {
        let mut w = make_doc("hello");
        w.cursor.set_raw(&w.content, 5);
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn newline_inserts_newline() {
        let mut w = make_doc("ab");
        w.cursor.set_raw(&w.content, 1);
        newline(&mut w);
        assert_eq!(w.content, "a\nb");
        assert_eq!(w.cursor.raw(), 2);
    }

    #[test]
    fn get_text_returns_content() {
        let w = make_doc("hello");
        assert_eq!(get_text(&w), "hello");
    }

    #[test]
    fn get_line_count_works() {
        let w = make_doc("a\nb\nc");
        assert_eq!(get_line_count(&w), 3);
    }

    #[test]
    fn get_line_works() {
        let w = make_doc("first\nsecond\nthird");
        assert_eq!(get_line(&w, 0), Some("first"));
        assert_eq!(get_line(&w, 1), Some("second"));
        assert_eq!(get_line(&w, 2), Some("third"));
        assert_eq!(get_line(&w, 3), None);
    }

    #[test]
    fn text_len_works() {
        let w = make_doc("hello");
        assert_eq!(text_len(&w), 5);
    }

    #[test]
    fn unicode_insert() {
        let mut w = make_doc("Приве");
        w.cursor.set_raw(&w.content, 10);
        insert_at_cursor(&mut w, "т");
        assert_eq!(w.content, "Привет");
        assert_eq!(w.cursor.raw(), 12);
    }

    #[test]
    fn unicode_delete_before() {
        let mut w = make_doc("Привет");
        w.cursor.set_raw(&w.content, 12);
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "Приве");
        assert_eq!(w.cursor.raw(), 10);
    }

    #[test]
    fn grapheme_delete_before() {
        let mut w = make_doc("e\u{0301}x");
        w.cursor.set_raw(&w.content, 4);
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "e\u{0301}");
    }

    #[test]
    fn grapheme_delete_after() {
        let mut w = make_doc("e\u{0301}x");
        w.cursor.set_raw(&w.content, 0);
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "x");
    }
}
