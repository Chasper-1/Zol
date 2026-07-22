use super::cursor_x::cursor_x_on_line;
use super::raw_at_x::raw_at_x_on_line;
use crate::editor::layout::cursor_line_bounds;
use crate::gui::iced_editor::EditorInner;

/// Переместить курсор на строку `target_line`, сохраняя пиксельную X.
pub fn move_vertical(inner: &EditorInner, target_line: usize) {
    let x = {
        let doc = inner.doc.borrow();
        let shaped = inner.shaped_doc.borrow();
        let cl = doc.cursor.line();
        let (ls, _) = cursor_line_bounds(&doc.content, cl);
        let byte_in_line = doc.cursor.raw().saturating_sub(ls);
        cursor_x_on_line(&shaped, cl, byte_in_line)
    };

    let new_raw = {
        let doc = inner.doc.borrow();
        let shaped = inner.shaped_doc.borrow();
        let (t_start, t_end) = cursor_line_bounds(&doc.content, target_line);
        raw_at_x_on_line(&shaped, target_line, x, t_start, t_end)
    };

    let content = inner.doc.borrow().content.clone();
    {
        let mut doc = inner.doc.borrow_mut();
        doc.cursor.set_raw(&content, new_raw);
        doc.cursor.set_col_visual(x);
        doc.dirty = true;
    }
}
