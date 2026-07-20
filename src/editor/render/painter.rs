//! Отрисовка сформованного документа через egui `Painter`.
//!
//! ВРЕМЕННЫЙ МОСТ: будет заменён на Iced-версию при переезде.
//! Единственное место в `editor/render/`, зависящее от egui.

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

use crate::editor::cursor::Cursor;
use crate::editor::layout::cursor_line_bounds;
use crate::editor::render::shape::ShapedDocument;
use crate::editor::state::EditMode;

/// Нарисовать документ.
pub fn paint(
    doc: &ShapedDocument,
    cursor: &Cursor,
    painter: &Painter,
    origin: Pos2,
    text_color: Color32,
    mode: EditMode,
    content: &str,
) {
    // Рисуем текст через итерацию layout_runs() — не требует &mut Buffer.
    // glyph.x / glyph.y — float-координаты внутри буфера (line_y — базовая линия).
    for run in doc.buffer.layout_runs() {
        for glyph in run.glyphs {
            let x = origin.x + glyph.x;
            let y = origin.y + run.line_y + glyph.y;
            let w = glyph.w;
            let h = glyph.font_size;

            let color32 = glyph.color_opt.map_or(text_color, |c| {
                Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), c.a())
            });

            painter.rect_filled(
                Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h)),
                0.0,
                color32,
            );
        }
    }

    // Курсор
    if mode != EditMode::Preview {
        draw_cursor(doc, cursor, painter, origin, text_color, content);
    }
}

fn draw_cursor(
    doc: &ShapedDocument,
    cursor: &Cursor,
    painter: &Painter,
    origin: Pos2,
    text_color: Color32,
    content: &str,
) {
    let (line_start, _) = cursor_line_bounds(content, cursor.line());
    let byte_in_line = cursor.raw().saturating_sub(line_start);

    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;
    let mut line_h = 12.0;

    for run in doc.buffer.layout_runs() {
        if run.line_i != cursor.line() {
            continue;
        }
        cursor_y = run.line_top;
        line_h = run.line_height;

        let mut found = false;
        for glyph in run.glyphs {
            if glyph.start >= byte_in_line {
                cursor_x = glyph.x;
                found = true;
                break;
            }
        }
        if !found {
            cursor_x = run.glyphs.last().map(|g| g.x + g.w).unwrap_or(0.0);
        }
        break;
    }

    let pos = Pos2::new(origin.x + cursor_x, origin.y + cursor_y);
    painter.line_segment(
        [pos, Pos2::new(pos.x, pos.y + line_h)],
        Stroke::new(2.0, text_color),
    );
}

/// Получить позицию клика в документе: возвращает (line, byte_offset).
pub fn click_position(
    doc: &ShapedDocument,
    local_pos: Pos2,
) -> Option<(usize, usize)> {
    let cosmic_cursor = doc.buffer.hit(local_pos.x, local_pos.y)?;
    Some((cosmic_cursor.line, cosmic_cursor.index))
}
