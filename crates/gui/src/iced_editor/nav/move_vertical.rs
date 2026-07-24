use super::cursor_x::cursor_x_on_line;
use super::raw_at_x::raw_at_x_on_line;
use editor::layout::cursor_line_bounds;
use crate::iced_editor::EditorInner;

/// Переместить курсор на строку `target_line`, сохраняя пиксельную X.
pub fn move_vertical(inner: &EditorInner, target_line: usize) {
    let x = {
        let doc = inner.doc.borrow();
        let shaped = inner.shaped_doc.borrow();
        let src: &str = &doc.incremental.source;
        let cl = doc.cursor.line();
        let (ls, _) = cursor_line_bounds(src, cl);
        let byte_in_line = doc.cursor.raw().saturating_sub(ls);
        cursor_x_on_line(&shaped, cl, byte_in_line)
    };

    let new_raw = {
        let doc = inner.doc.borrow();
        let shaped = inner.shaped_doc.borrow();
        let src: &str = &doc.incremental.source;
        let (t_start, t_end) = cursor_line_bounds(src, target_line);
        raw_at_x_on_line(&shaped, target_line, x, t_start, t_end)
    };

    {
        let mut doc = inner.doc.borrow_mut();
        doc.set_cursor_raw(new_raw);
        doc.cursor.set_col_visual(x);
        doc.dirty = true;
    }
}
