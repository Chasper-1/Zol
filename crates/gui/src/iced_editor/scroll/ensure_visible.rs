use editor::render::ShapedDocument;
use super::layout_y::layout_line_y;

/// Откорректировать `scroll_y` так, чтобы строка с курсором была видна.
pub fn ensure_cursor_visible(
    scroll_y: f32,
    viewport_height: f32,
    shaped: &ShapedDocument,
    cursor_line: usize,
) -> f32 {
    if viewport_height <= 0.0 {
        return scroll_y;
    }

    let cursor_y = layout_line_y(shaped, cursor_line);
    let line_h = shaped.line_height(cursor_line);

    if cursor_y < scroll_y {
        cursor_y
    } else if cursor_y + line_h > scroll_y + viewport_height {
        cursor_y + line_h - viewport_height
    } else {
        scroll_y
    }
}
