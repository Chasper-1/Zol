//! Отрисовка текста: каждый TextRun отдельно со своим шрифтом/цветом.

use iced::advanced::text::{self, Renderer as TextRenderer};
use iced::font::{Style as FontStyle, Weight};
use iced::{
    alignment, Color, Pixels, Point, Rectangle, Size,
};

use editor::markup::segment::{STYLE_BOLD, STYLE_ITALIC};

use crate::iced_editor::widget::editor::IcedEditor;

pub fn draw_text<'a, Renderer>(
    this: &IcedEditor<'a>,
    renderer: &mut Renderer,
    origin: Point,
) where
    Renderer: TextRenderer<Font = iced::Font>,
{
    let shaped = this.inner.shaped_doc.borrow();
    let scroll_y = this.inner.scroll_y.get();

    for run in shaped.buffer.layout_runs() {
        let line_top = run.line_top - scroll_y;
        if run.text == "\u{200B}" {
            continue;
        }

        let line_i = run.line_i;
        let text_runs = shaped.line_runs.get(line_i);

        if let Some(text_runs) = text_runs {
            let mut byte_offset = 0usize;

            for tr in text_runs {
                if tr.text.is_empty() || tr.text == "\u{200B}" {
                    continue;
                }

                let x_offset = run
                    .glyphs
                    .iter()
                    .find(|g| {
                        g.start >= byte_offset && g.start < byte_offset + tr.text.len()
                    })
                    .map(|g| g.x)
                    .unwrap_or(0.0);

                let mut font = iced::Font::DEFAULT;
                if tr.style_flags & STYLE_BOLD != 0 {
                    font.weight = Weight::Bold;
                }
                if tr.style_flags & STYLE_ITALIC != 0 {
                    font.style = FontStyle::Italic;
                }

                let color = Color::from_rgba8(
                    (tr.color.r * 255.0) as u8,
                    (tr.color.g * 255.0) as u8,
                    (tr.color.b * 255.0) as u8,
                    tr.color.a as f32,
                );

                renderer.fill_text(
                    text::Text {
                        content: tr.text.clone(),
                        bounds: Size::new(f32::INFINITY, run.line_height),
                        size: Pixels(tr.size),
                        line_height: text::LineHeight::Absolute(Pixels(run.line_height)),
                        font,
                        align_x: text::Alignment::Left,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(origin.x + x_offset, origin.y + line_top),
                    color,
                    Rectangle::new(origin, Size::new(f32::INFINITY, f32::INFINITY)),
                );

                byte_offset += tr.text.len();
            }
        }
    }
}
