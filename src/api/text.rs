use crate::editor::editor_widget::EditorWidget;

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

pub fn get_text(widget: &EditorWidget) -> &str {
    &widget.content
}

pub fn get_line(widget: &EditorWidget, idx: usize) -> Option<&str> {
    if widget.content.is_empty() {
        return if idx == 0 { Some("") } else { None };
    }
    let mut current = 0usize;
    let mut start = 0usize;
    for (i, c) in widget.content.char_indices() {
        if current == idx {
            if c == '\n' {
                return Some(&widget.content[start..i]);
            }
        }
        if c == '\n' {
            current += 1;
            start = i + 1;
        }
    }
    if current == idx {
        Some(&widget.content[start..])
    } else {
        None
    }
}

pub fn get_line_count(widget: &EditorWidget) -> usize {
    if widget.content.is_empty() {
        return 1;
    }
    widget.content.chars().filter(|&c| c == '\n').count() + 1
}

pub fn text_len(widget: &EditorWidget) -> usize {
    widget.content.len()
}
