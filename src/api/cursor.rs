#![allow(dead_code)]

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

pub fn prev_char(content: &str, from: usize) -> usize {
    if from == 0 || content.is_empty() {
        return 0;
    }
    let from = from.min(content.len());
    content[..from].char_indices().last().map(|(i, _)| i).unwrap_or(0)
}

pub fn next_char(content: &str, from: usize) -> usize {
    let len = content.len();
    let from = from.min(len);
    if from >= len {
        return len;
    }
    if let Some((n, _)) = content[from..].char_indices().nth(1) {
        from + n
    } else {
        len
    }
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

pub fn next_word_end(content: &str, from: usize) -> usize {
    let len = content.len();
    if from >= len {
        return len;
    }

    let bytes = content.as_bytes();
    let mut pos = next_word_start(content, from);

    // Skip to end of the word we landed at
    while pos < len && !is_space_or_newline(bytes[pos]) {
        pos += 1;
    }

    pos
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

fn is_space_or_newline(b: u8) -> bool {
    b == b' ' || b == b'\n'
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
