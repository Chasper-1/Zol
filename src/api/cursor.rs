use crate::editor::editor_widget::EditorWidget;
use crate::editor::line_utils;

pub fn move_left(widget: &mut EditorWidget) {
    widget.cursor.move_left(&widget.content);
}

pub fn move_right(widget: &mut EditorWidget) {
    widget.cursor.move_right(&widget.content);
}

pub fn move_up(widget: &mut EditorWidget) {
    let line = widget.cursor.line;
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
    widget.cursor.raw = start + target_pos;
    widget.cursor.line = prev_line;
    widget.cursor.set_col_visual(col_x);
}

pub fn move_down(widget: &mut EditorWidget) {
    let line = widget.cursor.line;
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
    widget.cursor.raw = start + target_pos;
    widget.cursor.line = next_line;
    widget.cursor.set_col_visual(col_x);
}

pub fn move_home(widget: &mut EditorWidget) {
    widget.cursor.move_home(&widget.content);
}

pub fn move_end(widget: &mut EditorWidget) {
    widget.cursor.move_end(&widget.content);
}

pub fn move_word_left(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw;
    let pos = prev_word_start(&widget.content, raw);
    widget.cursor.raw = pos;
    widget.cursor.update_line(&widget.content);
    widget.cursor.reset_col_visual();
}

pub fn move_word_right(widget: &mut EditorWidget) {
    let raw = widget.cursor.raw;
    let pos = next_word_start(&widget.content, raw);
    widget.cursor.raw = pos;
    widget.cursor.update_line(&widget.content);
    widget.cursor.reset_col_visual();
}

pub fn prev_word_start(content: &str, from: usize) -> usize {
    if from == 0 || content.is_empty() {
        return 0;
    }
    let pos = from.min(content.len());

    let bytes = content.as_bytes();

    // 1. Skip whitespace/newline backward
    let mut i = pos;
    while i > 0 && is_space_or_newline(bytes[i - 1]) {
        i -= 1;
    }

    if i == 0 {
        return 0;
    }

    // 2. Skip word backward to its start
    let mut j = i;
    while j > 0 && !is_space_or_newline(bytes[j - 1]) {
        j -= 1;
    }

    // If we found a word boundary (moved), return it
    if j < i {
        return j;
    }

    // Already at word start. Move to start of previous word.
    // Skip current word backward
    while i > 0 && !is_space_or_newline(bytes[i - 1]) {
        i -= 1;
    }
    // Skip whitespace backward
    while i > 0 && is_space_or_newline(bytes[i - 1]) {
        i -= 1;
    }
    // Skip to start of previous word
    while i > 0 && !is_space_or_newline(bytes[i - 1]) {
        i -= 1;
    }
    i
}

pub fn next_word_start(content: &str, from: usize) -> usize {
    let len = content.len();
    if from >= len {
        return len;
    }

    let bytes = content.as_bytes();
    let mut pos = from;

    // 1. If on word char, skip to end of word
    if !is_space_or_newline(bytes[pos]) {
        while pos < len && !is_space_or_newline(bytes[pos]) {
            pos += 1;
        }
    }

    // 2. Skip whitespace/newline to find start of next word
    while pos < len && is_space_or_newline(bytes[pos]) {
        pos += 1;
    }

    pos
}

fn is_space_or_newline(b: u8) -> bool {
    b == b' ' || b == b'\n'
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

    #[test]
    fn move_left_basic() {
        let mut w = make_widget("abc");
        w.cursor.raw = 2;
        move_left(&mut w);
        assert_eq!(w.cursor.raw, 1);
        assert_eq!(w.cursor.line, 0);
    }

    #[test]
    fn move_left_at_start() {
        let mut w = make_widget("abc");
        move_left(&mut w);
        assert_eq!(w.cursor.raw, 0);
    }

    #[test]
    fn move_right_basic() {
        let mut w = make_widget("abc");
        move_right(&mut w);
        assert_eq!(w.cursor.raw, 1);
    }

    #[test]
    fn move_right_at_end() {
        let mut w = make_widget("abc");
        w.cursor.raw = 3;
        move_right(&mut w);
        assert_eq!(w.cursor.raw, 3);
    }

    #[test]
    fn move_up_simple() {
        let mut w = make_widget("first\nsecond");
        w.cursor.raw = 10; // somewhere in "second"
        w.cursor.update_line(&w.content);
        move_up(&mut w);
        assert_eq!(w.cursor.line, 0);
    }

    #[test]
    fn move_up_at_first_line_goes_home() {
        let mut w = make_widget("only one line");
        w.cursor.raw = 5;
        move_up(&mut w);
        assert_eq!(w.cursor.raw, 0); // goes to home
    }

    #[test]
    fn move_down_simple() {
        let mut w = make_widget("first\nsecond");
        move_down(&mut w);
        assert_eq!(w.cursor.line, 1);
    }

    #[test]
    fn move_down_at_last_line_goes_to_end() {
        let mut w = make_widget("first\nsecond");
        w.cursor.raw = 6; // start of "second"
        w.cursor.line = 1; // already on last line
        let len_before = w.content.len();
        move_down(&mut w);
        assert_eq!(w.cursor.raw, len_before); // goes to end of content
    }

    #[test]
    fn move_home_goes_to_start() {
        let mut w = make_widget("hello world");
        w.cursor.raw = 5;
        move_home(&mut w);
        assert_eq!(w.cursor.raw, 0);
    }

    #[test]
    fn move_end_goes_to_end() {
        let mut w = make_widget("hello world");
        move_end(&mut w);
        assert_eq!(w.cursor.raw, 11);
    }

    #[test]
    fn move_word_left_works() {
        let mut w = make_widget("hello world foo");
        w.cursor.raw = 16; // after "foo"
        w.cursor.update_line(&w.content);
        move_word_left(&mut w);
        assert_eq!(w.cursor.raw, 12); // start of "foo"
    }

    #[test]
    fn move_word_right_works() {
        let mut w = make_widget("hello world");
        move_word_right(&mut w);
        assert_eq!(w.cursor.raw, 6); // start of "world"
    }

    #[test]
    fn prev_word_start_works() {
        let content = "abc def ghi";
        assert_eq!(prev_word_start(content, 11), 8); // start of "ghi"
        assert_eq!(prev_word_start(content, 7), 4); // start of "def"
        assert_eq!(prev_word_start(content, 3), 0); // start of "abc"
        assert_eq!(prev_word_start(content, 0), 0); // already at start
    }

    #[test]
    fn next_word_start_works() {
        let content = "abc def ghi";
        assert_eq!(next_word_start(content, 0), 4); // start of "def"
        assert_eq!(next_word_start(content, 4), 8); // start of "ghi"
        assert_eq!(next_word_start(content, 11), 11); // at/after end
    }

    #[test]
    fn unicode_move_left_right() {
        let mut w = make_widget("Привет");
        w.cursor.raw = 12; // end of string
        move_left(&mut w);
        assert_eq!(w.cursor.raw, 10); // before "т"
        move_right(&mut w);
        assert_eq!(w.cursor.raw, 12); // back to end
    }
}
