pub mod text;
pub mod cursor;

use iced::advanced::renderer;
use iced::Color;
use iced::{Point, Rectangle};

use super::editor::IcedEditor;

/// Точка входа для `Widget::draw()`.
pub fn draw<'a, Renderer>(
    this: &IcedEditor<'a>,
    renderer: &mut Renderer,
    layout: iced::advanced::Layout<'_>,
    _mouse_cursor: iced::advanced::mouse::Cursor,
) where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    let bounds = layout.bounds();
    this.last_bounds.set(bounds);
    let origin = Point::new(bounds.x, bounds.y);

    // ── Фаза 1: перешейп ─────────────────────────────────────────────
    draw_reshape(this, bounds);

    // ── Фаза 2: фон ───────────────────────────────────────────────────
    draw_background(this, renderer, bounds);

    // ── Фаза 3: текст ────────────────────────────────────────────────
    text::draw_text(this, renderer, origin);

    // ── Фаза 4: курсор ────────────────────────────────────────────────
    cursor::draw_cursor(this, renderer, origin);
}

/// Перестроить shaped-буфер, если контент изменился.
fn draw_reshape(this: &IcedEditor<'_>, bounds: Rectangle) {
    let needs_reshape = {
        let doc = this.inner.doc.borrow();
        doc.dirty
    };

    if needs_reshape {
        // Обновляем viewport из текущего scroll_y и высоты виджета
        let vp = this.inner.compute_viewport(bounds.height);
        this.inner.viewport.set(vp);

        // Заимствуем всё, что нужно для render::build, без клонирования
        let doc = this.inner.doc.borrow();
        let content: &str = doc.content();
        let cursor_line = doc.cursor.line();
        let scroll_y = this.inner.scroll_y.get();
        let mode = this.inner.get_mode();
        let theme = &this.inner.theme;
        let cache = this.inner.cache.borrow();
        let mut shaped = this.inner.shaped_doc.borrow_mut();

        let vp = this.inner.viewport.get();
        editor::render::build(
            &mut *shaped,
            content,
            &cache,
            mode,
            cursor_line,
            theme,
            this.inner.base_size,
            this.inner.heading_size,
            scroll_y,
            Some(bounds.height),
            Some(&vp),
        );
        // Все заимствования (doc, cache, shaped) завершаются здесь
        drop(shaped);
        drop(cache);
        drop(doc);
        this.inner.doc.borrow_mut().dirty = false;
    }
}

/// Заливка фона редактора.
fn draw_background<Renderer>(this: &IcedEditor<'_>, renderer: &mut Renderer, bounds: Rectangle)
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
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
}
