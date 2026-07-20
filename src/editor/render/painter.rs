use crate::editor::cursor::Cursor;
use super::layout::cursor_line_bounds;
use crate::editor::state::EditMode;
use crate::editor::utils::line_utils;
use super::Galleys;
use eframe::egui::text::{CCursor, Galley};
use eframe::egui::{Color32, Painter, Pos2, Stroke};

pub fn paint(
    galleys: &Galleys,
    cursor: &Cursor,
    painter: &Painter,
    origin: Pos2,
    text_color: Color32,
    content: &str,
    mode: EditMode,
) {
    let mut y_offset = origin.y;

    for (i, galley_opt) in galleys.galleys.iter().enumerate() {
        if let Some(galley) = galley_opt {
            let galley_size = galley.size();
            let pos = Pos2::new(origin.x, y_offset);

            painter.galley(pos, galley.clone(), text_color);

            if mode != EditMode::Preview
                && i == cursor.line
                && let Some(cursor_rect) = cursor_rect(content, cursor, galley)
            {
                let cursor_x = origin.x + cursor_rect.min.x;
                let cursor_y = y_offset + cursor_rect.min.y;
                let line_h = cursor_rect.height().max(galley_size.y * 0.8);

                painter.line_segment(
                    [
                        Pos2::new(cursor_x, cursor_y),
                        Pos2::new(cursor_x, cursor_y + line_h),
                    ],
                    Stroke::new(2.0, text_color),
                );
            }

            y_offset += galley_size.y;
        }
    }
}

fn cursor_rect(content: &str, cursor: &Cursor, galley: &Galley) -> Option<eframe::egui::Rect> {
    let (line_start, line_end) = cursor_line_bounds(content, cursor.line);
    let byte_in_line = cursor.raw.saturating_sub(line_start);
    let line_text = line_utils::safe_slice(content, line_start, line_end);
    let byte_in_line = byte_in_line.min(line_text.len());
    let safe_byte = if line_text.is_char_boundary(byte_in_line) {
        byte_in_line
    } else {
        let mut b = byte_in_line;
        while b > 0 && !line_text.is_char_boundary(b) {
            b -= 1;
        }
        b
    };
    let char_idx = line_text[..safe_byte].chars().count();
    let egui_cursor = CCursor::new(char_idx);
    Some(galley.pos_from_cursor(egui_cursor))
}
