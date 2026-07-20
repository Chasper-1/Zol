use crate::editor::editor_widget::EditorWidget;
use crate::editor::utils::line_utils;

pub fn move_left(widget: &mut EditorWidget) {
    widget.cursor.move_left(&widget.content);
}

pub fn move_right(widget: &mut EditorWidget) {
    widget.cursor.move_right(&widget.content);
}

pub fn move_up(widget: &mut EditorWidget) {
    let line = widget.cursor.line();
    if line == 0 {
        widget.cursor.move_home(&widget.content);
        return;
    }

    let col_x = widget.cursor.col_visual();
    let prev_line = line - 1;
    let prev_text = line_utils::line_text(&widget.content, prev_line).unwrap_or("");
    let target_pos = if col_x.is_infinite() {
        prev_text.len()
    } else {
        x_to_char_pos(prev_text, col_x)
    };

    let start = line_utils::line_start_byte(&widget.content, prev_line);
    widget.cursor.set_raw(&widget.content, start + target_pos);
    widget.cursor.set_line(prev_line);
    widget.cursor.set_col_visual(col_x);
}

pub fn move_down(widget: &mut EditorWidget) {
    let line = widget.cursor.line();
    let total = line_utils::count_lines(&widget.content);

    if line + 1 >= total {
        widget.cursor.move_end(&widget.content);
        return;
    }

    let col_x = widget.cursor.col_visual();
    let next_line = line + 1;
    let next_text = line_utils::line_text(&widget.content, next_line).unwrap_or("");
    let target_pos = if col_x.is_infinite() {
        next_text.len()
    } else {
        x_to_char_pos(next_text, col_x)
    };

    let start = line_utils::line_start_byte(&widget.content, next_line);
    widget.cursor.set_raw(&widget.content, start + target_pos);
    widget.cursor.set_line(next_line);
    widget.cursor.set_col_visual(col_x);
}

pub fn move_home(widget: &mut EditorWidget) {
    widget.cursor.move_home(&widget.content);
}

pub fn move_end(widget: &mut EditorWidget) {
    widget.cursor.move_end(&widget.content);
}

pub fn move_word_left(widget: &mut EditorWidget) {
    widget.cursor.move_word_left(&widget.content);
}

pub fn move_word_right(widget: &mut EditorWidget) {
    widget.cursor.move_word_right(&widget.content);
}

fn x_to_char_pos(line: &str, x: f32) -> usize {
    let char_count = line.chars().count();
    let approx = (x / 10.0).round() as usize;
    approx.min(char_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::editor_widget::EditorWidget;

    fn make_widget(text: &str) -> EditorWidget {
        EditorWidget::new(text)
    }

    // ── move_left/move_right ──────────────────────────────────

    #[test]
    fn move_left_basic() {
        let mut w = make_widget("abc");
        w.cursor.set_raw(&w.content, 2);
        move_left(&mut w);
        assert_eq!(w.cursor.raw(), 1);
        assert_eq!(w.cursor.line(), 0);
    }

    #[test]
    fn move_left_at_start() {
        let mut w = make_widget("abc");
        move_left(&mut w);
        assert_eq!(w.cursor.raw(), 0);
    }

    #[test]
    fn move_right_basic() {
        let mut w = make_widget("abc");
        move_right(&mut w);
        assert_eq!(w.cursor.raw(), 1);
    }

    #[test]
    fn move_right_at_end() {
        let mut w = make_widget("abc");
        w.cursor.set_raw(&w.content, 3);
        move_right(&mut w);
        assert_eq!(w.cursor.raw(), 3);
    }

    // ── grapheme-навигация ────────────────────────────────────

    #[test]
    fn grapheme_move_left_from_mid() {
        let mut w = make_widget("e\u{0301}x");
        w.cursor.set_raw(&w.content, 3);
        move_left(&mut w);
        assert_eq!(w.cursor.raw(), 0);
    }

    #[test]
    fn grapheme_move_left_from_end() {
        let mut w = make_widget("e\u{0301}x");
        w.cursor.set_raw(&w.content, 4);
        move_left(&mut w);
        assert_eq!(w.cursor.raw(), 3);
        move_left(&mut w);
        assert_eq!(w.cursor.raw(), 0);
    }

    #[test]
    fn grapheme_move_right() {
        let mut w = make_widget("e\u{0301}x");
        move_right(&mut w);
        assert_eq!(w.cursor.raw(), 3);
        move_right(&mut w);
        assert_eq!(w.cursor.raw(), 4);
    }

    // ── move_up/move_down ─────────────────────────────────────

    #[test]
    fn move_up_simple() {
        let mut w = make_widget("first\nsecond");
        w.cursor.set_raw(&w.content, 10);
        move_up(&mut w);
        assert_eq!(w.cursor.line(), 0);
    }

    #[test]
    fn move_up_at_first_line_goes_home() {
        let mut w = make_widget("only one line");
        w.cursor.set_raw(&w.content, 5);
        move_up(&mut w);
        assert_eq!(w.cursor.raw(), 0);
    }

    #[test]
    fn move_down_simple() {
        let mut w = make_widget("first\nsecond");
        move_down(&mut w);
        assert_eq!(w.cursor.line(), 1);
    }

    #[test]
    fn move_down_at_last_line_goes_to_end() {
        let mut w = make_widget("first\nsecond");
        w.cursor.set_raw(&w.content, 6);
        w.cursor.set_line(1);
        let len_before = w.content.len();
        move_down(&mut w);
        assert_eq!(w.cursor.raw(), len_before);
    }

    // ── move_home/move_end ────────────────────────────────────

    #[test]
    fn move_home_goes_to_start() {
        let mut w = make_widget("hello world");
        w.cursor.set_raw(&w.content, 5);
        move_home(&mut w);
        assert_eq!(w.cursor.raw(), 0);
    }

    #[test]
    fn move_end_goes_to_end() {
        let mut w = make_widget("hello world");
        move_end(&mut w);
        assert_eq!(w.cursor.raw(), 11);
    }

    // ── move_word ─────────────────────────────────────────────

    #[test]
    fn move_word_left_works() {
        let mut w = make_widget("hello world foo");
        w.cursor.set_raw(&w.content, 16);
        move_word_left(&mut w);
        assert_eq!(w.cursor.raw(), 12);
    }

    #[test]
    fn move_word_right_works() {
        let mut w = make_widget("hello world");
        move_word_right(&mut w);
        assert_eq!(w.cursor.raw(), 6);
    }

    // ── unicode ───────────────────────────────────────────────

    #[test]
    fn unicode_move_left_right() {
        let mut w = make_widget("Привет");
        w.cursor.set_raw(&w.content, 12);
        move_left(&mut w);
        assert_eq!(w.cursor.raw(), 10);
        move_right(&mut w);
        assert_eq!(w.cursor.raw(), 12);
    }
}
