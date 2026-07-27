//! Отрисовка выделения текста.
//!
//! Проходит по глифам cosmic-text, определяет байтовый диапазон каждого
//! глифа (от его `glyph.start` до `start` следующего глифа) и рисует
//! прямоугольник, если глиф попадает в выделение.

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
        let mut glyphs = run.glyphs.iter().peekable();

        while let Some(glyph) = glyphs.next() {
            // Байтовый конец глифа = start следующего глифа (или конец строки, если последний)
            let glyph_byte_end = glyphs.peek().map(|next| next.start).unwrap_or(sel_end); // за пределами выделения

            // Нет пересечения с [sel_start, sel_end)
            if glyph_byte_end <= sel_start || glyph.start >= sel_end {
                continue;
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(origin.x + glyph.x, origin.y + line_top),
                        Size::new(glyph.w, run.line_height),
                    ),
                    ..renderer::Quad::default()
                },
                iced::Background::Color(sel_color),
            );
        }
    }
}
