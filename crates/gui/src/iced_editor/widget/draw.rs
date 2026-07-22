//! Отрисовка текста и курсора.
//!
//! Рисует:
//! - Фон редактора (однотонная заливка).
//! - Текст: для каждого TextRun свой `fill_text` с правильным
//!   шрифтом (bold/italic) и цветом.
//! - Курсор: вертикальная полоска (в режимах Preview не рисуется).

use iced::advanced::renderer;
use iced::advanced::text::{self, Renderer as TextRenderer};
use iced::advanced::{mouse};
use iced::{
    alignment, Color, Pixels, Point, Rectangle, Size,
};
use iced::font::{Style as FontStyle, Weight};

use editor::layout::cursor_line_bounds;
use editor::markup::segment::{STYLE_BOLD, STYLE_ITALIC};
use editor::render;
use editor::state::EditMode;

use super::editor::IcedEditor;

/// Точка входа для `Widget::draw()`.
pub fn draw<'a, Renderer>(
    this: &IcedEditor<'a>,
    renderer: &mut Renderer,
    layout: iced::advanced::Layout<'_>,
    _mouse_cursor: mouse::Cursor,
) where
    Renderer: TextRenderer<Font = iced::Font>,
{
    let bounds = layout.bounds();
    this.last_bounds.set(bounds);
    let origin = Point::new(bounds.x, bounds.y);

    // ── Фаза 1: перешейп (mutable borrow shaped_doc) ──────────────
    // Забираем всё, что нужно из doc, до borrow_mut shaped_doc
    let needs_reshape = {
        let doc = this.inner.doc.borrow();
        doc.dirty
    };

    if needs_reshape {
        let (content, cursor_line, scroll_y) = {
            let doc = this.inner.doc.borrow();
            (doc.content.clone(), doc.cursor.line(), this.inner.scroll_y.get())
        };
        let mode = this.inner.get_mode();
        let theme = &this.inner.theme;
        let cache = this.inner.cache.borrow().clone();
        let mut shaped = this.inner.shaped_doc.borrow_mut();
        render::build(
            &mut *shaped,
            &content,
            &cache,
            mode,
            cursor_line,
            theme,
            this.inner.base_size,
            this.inner.heading_size,
            scroll_y,
            Some(bounds.height),
        );
        drop(shaped);
        this.inner.doc.borrow_mut().dirty = false;
    }

    // ── Фаза 2: фон ───────────────────────────────────────────────
    let bg = &this.inner.theme.background;
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            ..renderer::Quad::default()
        },
        iced::Background::Color(Color::from_rgba8(
            (bg.r * 255.0) as u8,
            (bg.g * 255.0) as u8,
            (bg.b * 255.0) as u8,
            bg.a as f32,
        )),
    );

    // ── Фаза 3: текст — каждый TextRun отдельно (bold / italic / цвет) ──
    let shaped = this.inner.shaped_doc.borrow();
    let scroll_y = this.inner.scroll_y.get();

    for run in shaped.buffer.layout_runs() {
        let line_top = run.line_top - scroll_y;
        // Пропускаем zero-width space (пустые строки)
        if run.text == "\u{200B}" {
            continue;
        }

        let line_i = run.line_i;
        let text_runs = shaped.line_runs.get(line_i);

        if let Some(text_runs) = text_runs {
            // byte_offset отслеживает позицию в строке для поиска глифов
            let mut byte_offset = 0usize;

            for tr in text_runs {
                if tr.text.is_empty() || tr.text == "\u{200B}" {
                    continue;
                }

                // X offset из глифов cosmic-text — находим первый глиф,
                // принадлежащий этому TextRun'у
                let x_offset = run
                    .glyphs
                    .iter()
                    .find(|g| {
                        g.start >= byte_offset && g.start < byte_offset + tr.text.len()
                    })
                    .map(|g| g.x)
                    .unwrap_or(0.0);

                // Формируем шрифт с весом и стилем
                let mut font = iced::Font::DEFAULT;
                if tr.style_flags & STYLE_BOLD != 0 {
                    font.weight = Weight::Bold;
                }
                if tr.style_flags & STYLE_ITALIC != 0 {
                    font.style = FontStyle::Italic;
                }

                // Цвет из TextRun
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
                    Rectangle::new(origin, bounds.size()),
                );

                byte_offset += tr.text.len();
            }
        }
    }
    drop(shaped);

    // ── Фаза 4: курсор (только в Edit/Preview режимах) ───────────
    if this.inner.get_mode() != EditMode::Preview {
        let (cursor_line, cursor_raw, should_blink) = {
            let doc = this.inner.doc.borrow();
            (doc.cursor.line(), doc.cursor.raw(), doc.cursor.should_blink())
        };

        if should_blink {
            let shaped = this.inner.shaped_doc.borrow();
            let content = this.inner.doc.borrow().content.clone();
            let (line_start, _) = cursor_line_bounds(&content, cursor_line);
            let byte_in_line = cursor_raw.saturating_sub(line_start);

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
                    // Если курсор за последним глифом — ставим в конец строки
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
    }
}
