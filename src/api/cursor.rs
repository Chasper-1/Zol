use crate::editor::editor_widget::EditorWidget;

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

    let prev_text = line_text(&widget.content, prev_line);
    let target_pos = if col_x.is_infinite() {
        prev_text.len()
    } else {
        x_to_char_pos(prev_text, col_x)
    };

    let start = line_start_byte(&widget.content, prev_line);
    widget.cursor.raw = start + target_pos;
    widget.cursor.line = prev_line;
    widget.cursor.set_col_visual(col_x);
}

pub fn move_down(widget: &mut EditorWidget) {
    let line = widget.cursor.line;
    let total = line_count(&widget.content);

    if line + 1 >= total {
        widget.cursor.move_end(&widget.content);
        return;
    }

    let col_x = widget.cursor.col_visual();
    let next_line = line + 1;

    let next_text = line_text(&widget.content, next_line);
    let target_pos = if col_x.is_infinite() {
        next_text.len()
    } else {
        x_to_char_pos(next_text, col_x)
    };

    let start = line_start_byte(&widget.content, next_line);
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

pub fn cursor_pos(widget: &EditorWidget) -> usize {
    widget.cursor.raw
}

pub fn cursor_line(widget: &EditorWidget) -> usize {
    widget.cursor.line
}

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    content.chars().filter(|&c| c == '\n').count() + 1
}

fn line_text<'a>(content: &'a str, line: usize) -> &'a str {
    let mut current = 0usize;
    let mut start = 0usize;
    for (i, c) in content.char_indices() {
        if current == line {
            if c == '\n' {
                return &content[start..i];
            }
        }
        if c == '\n' {
            current += 1;
            start = i + 1;
        }
    }
    if current == line {
        &content[start..]
    } else {
        ""
    }
}

fn line_start_byte(content: &str, line: usize) -> usize {
    let mut current = 0usize;
    for (i, c) in content.char_indices() {
        if current == line {
            return i;
        }
        if c == '\n' {
            current += 1;
        }
    }
    if current == line && !content.is_empty() {
        content.len()
    } else {
        0
    }
}

fn x_to_char_pos(line: &str, x: f32) -> usize {
    let char_count = line.chars().count();
    let approx = (x / 10.0).round() as usize;
    approx.min(char_count)
}
