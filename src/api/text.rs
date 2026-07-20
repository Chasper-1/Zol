use crate::editor::editor_widget::EditorWidget;
use crate::editor::line_utils;

pub fn insert_at_cursor(widget: &mut EditorWidget, text: &str) {
    let raw = widget.cursor.raw;
    if raw > widget.content.len() {
        return;
    }
    widget.content.insert_str(raw, text);
    widget.cursor.raw = raw + text.len();
    widget.cursor.update_line(&widget.content);
    widget.cursor.force_blink_on();
    widget.dirty = true;
}

pub fn delete_before_cursor(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw;
    if raw == 0 || widget.content.is_empty() {
        return;
    }
    let prev = if let Some((idx, _)) = widget.content[..raw].char_indices().last() {
        idx
    } else {
        0
    };
    widget.content.drain(prev..raw);
    widget.cursor.raw = prev;
    widget.cursor.update_line(&widget.content);
    widget.cursor.force_blink_on();
    widget.dirty = true;
}

pub fn delete_after_cursor(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw;
    if raw >= widget.content.len() || widget.content.is_empty() {
        return;
    }
    let next = raw
        + if let Some((n, _)) = widget.content[raw..].char_indices().nth(1) {
            n
        } else {
            widget.content.len() - raw
        };
    widget.content.drain(raw..next);
    widget.cursor.update_line(&widget.content);
    widget.cursor.force_blink_on();
    widget.dirty = true;
}

pub fn newline(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw;
    if raw > widget.content.len() {
        return;
    }
    widget.content.insert(raw, '\n');
    widget.cursor.raw = raw + 1;
    widget.cursor.update_line(&widget.content);
    widget.cursor.reset_col_visual();
    widget.cursor.force_blink_on();
    widget.dirty = true;
}

// Публичный API для будущих плагинов и интеграций
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
        w.cursor.raw = 5; // move to end first
        insert_at_cursor(&mut w, " world");
        assert_eq!(w.content, "hello world");
        assert_eq!(w.cursor.raw, "hello world".len());
    }

    #[test]
    fn insert_at_cursor_mid_text() {
        let mut w = make_widget("helo");
        w.cursor.raw = 3;
        insert_at_cursor(&mut w, "l");
        assert_eq!(w.content, "hello");
    }

    #[test]
    fn insert_at_cursor_out_of_bounds() {
        let mut w = make_widget("hi");
        w.cursor.raw = 100;
        insert_at_cursor(&mut w, "!");
        assert_eq!(w.content, "hi"); // no change
    }

    #[test]
    fn delete_before_cursor_removes_char() {
        let mut w = make_widget("hello");
        w.cursor.raw = 5;
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "hell");
        assert_eq!(w.cursor.raw, 4);
    }

    #[test]
    fn delete_before_cursor_at_start() {
        let mut w = make_widget("hello");
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "hello"); // no change
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
        w.cursor.raw = 5;
        delete_after_cursor(&mut w);
        assert_eq!(w.content, "hello"); // no change
    }

    #[test]
    fn newline_inserts_newline() {
        let mut w = make_widget("ab");
        w.cursor.raw = 1;
        newline(&mut w);
        assert_eq!(w.content, "a\nb");
        assert_eq!(w.cursor.raw, 2);
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
        let mut w = make_widget("Приве"); // 5 chars, 10 bytes
        w.cursor.raw = 10; // end of string
        insert_at_cursor(&mut w, "т");
        assert_eq!(w.content, "Привет");
        assert_eq!(w.cursor.raw, 12);
    }

    #[test]
    fn unicode_delete_before() {
        let mut w = make_widget("Привет");
        w.cursor.raw = 12; // past "Приве" (10 bytes) + "т" (2 bytes)
        delete_before_cursor(&mut w);
        assert_eq!(w.content, "Приве"); // last char "т" removed
        assert_eq!(w.cursor.raw, 10);
    }
}
