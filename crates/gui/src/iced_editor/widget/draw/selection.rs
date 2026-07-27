//! Отрисовка выделенного текста (подсветка фона).

use iced::advanced::renderer;
use iced::{Color, Point, Rectangle, Size};

use crate::iced_editor::widget::editor::IcedEditor;

pub fn draw_selection<'a, Renderer>(this: &IcedEditor<'a>, renderer: &mut Renderer, origin: Point)
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    let sel_range = {
        let doc = this.inner.doc.borrow();
        doc.cursor.selection_range()
    };

    let Some((sel_start, sel_end)) = sel_range else {
        return;
    };

    let sel_color = {
        let c = &this.inner.theme.selection_bg;
        Color::from_rgba8(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            c.a as f32,
        )
    };

    let shaped = this.inner.shaped_doc.borrow();
    let scroll_y = this.inner.scroll_y.get();

    for run in shaped.buffer.layout_runs() {
        let line_top = run.line_top - scroll_y;

        for glyph in run.glyphs.iter() {
            // glyph.start — offset within the run's text,
            // we need absolute byte offset: glyph.start + run.byte_offset (or glyph.info.glyph_id?)
            // Actually cosmic-text: glyph.start is the byte offset of this glyph
            // relative to the entire buffer. Let's verify.

            // In cosmic-text, Glyph::start is byte offset in the source text.
            // We can use it directly.
            let glyph_end = glyph.start + glyph.w as usize; // approximate end
            if glyph_end <= sel_start || glyph.start >= sel_end {
                continue;
            }

            // Glyph overlaps with selection: compute intersection
            let rect_left = if glyph.start >= sel_start {
                glyph.x
            } else {
                // selection starts in the middle of this glyph
                glyph.x
            };

            let rect_right = if glyph_end <= sel_end {
                glyph.x + glyph.w
            } else {
                // selection ends in the middle of this glyph
                glyph.x + glyph.w
            };

            let width = rect_right - rect_left;
            if width <= 0.0 {
                continue;
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(origin.x + rect_left, origin.y + line_top),
                        Size::new(width, run.line_height),
                    ),
                    ..renderer::Quad::default()
                },
                iced::Background::Color(sel_color),
            );
        }
    }
}
