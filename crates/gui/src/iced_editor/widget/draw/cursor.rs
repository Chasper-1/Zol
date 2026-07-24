//! Отрисовка курсора: вертикальная полоска (кроме режима Preview).

use iced::advanced::renderer;
use iced::{Color, Point, Rectangle, Size};

use editor::layout::cursor_line_bounds;
use editor::state::EditMode;

use super::IcedEditor;

pub fn draw_cursor<'a, Renderer>(
    this: &IcedEditor<'a>,
    renderer: &mut Renderer,
    origin: Point,
) where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    if this.inner.get_mode() == EditMode::Preview {
        return;
    }

    let (cursor_line, cursor_raw, should_blink) = {
        let doc = this.inner.doc.borrow();
        (doc.cursor.line(), doc.cursor.raw(), doc.cursor.should_blink())
    };

    if !should_blink {
        return;
    }

    let shaped = this.inner.shaped_doc.borrow();
    let doc = this.inner.doc.borrow();
    let content: &str = &doc.incremental.source;
    let (line_start, _) = cursor_line_bounds(content, cursor_line);
    let byte_in_line = cursor_raw.saturating_sub(line_start);
    let scroll_y = this.inner.scroll_y.get();

    let (cursor_x, cursor_y, line_h) = {
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut lh = 12.0;

        for run in shaped.buffer.layout_runs() {
            if run.line_i != cursor_line {
                continue;
            }
            cy = run.line_top - scroll_y;
            lh = run.line_height;

            for glyph in run.glyphs.iter() {
                if glyph.start >= byte_in_line {
                    cx = glyph.x;
                    break;
                }
            }
            if cx == 0.0 {
                cx = run
                    .glyphs
                    .last()
                    .map(|g| g.x + g.w)
                    .unwrap_or(0.0);
            }
            break;
        }

        (cx, cy, lh)
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle::new(
                Point::new(origin.x + cursor_x, origin.y + cursor_y),
                Size::new(2.0, line_h),
            ),
            ..renderer::Quad::default()
        },
        Color::WHITE,
    );
}
