//! Iced-виджет редактора.
//!
//! Содержит [`IcedEditor`] — виджет, реализующий `Widget<Message, Theme, Renderer>`.
//! Рисует текст через GPU (`renderer.fill_text`), курсор — вертикальной полоской.
//! Позиционирование глифов и hit-testing — через cosmic-text.
//!
//! TODO:
//!   - Выделение (selection) через fill_quad с фоном
//!   - Цветная разметка (bold, italic, code…)
//!   - Автоскролл курсора
//!   - Мигание курсора по таймеру

use std::cell::Cell;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::advanced::text::{self};
use iced::{
    alignment, Color, Element, Event, Length, Pixels, Point, Rectangle, Size,
};

use crate::editor::cursor;
use crate::editor::layout::cursor_line_bounds;
use crate::editor::render;
use crate::editor::state::EditMode;

use super::inner::EditorInner;

// ---------------------------------------------------------------------------
// Виджет (держит &EditorInner — interior mutability через RefCell-поля)
// ---------------------------------------------------------------------------

/// Iced-виджет редактора.
pub struct IcedEditor<'a> {
    inner: &'a EditorInner,
    last_bounds: Cell<Rectangle>,
}

impl<'a> IcedEditor<'a> {
    pub fn new(inner: &'a EditorInner) -> Self {
        Self {
            inner,
            last_bounds: Cell::new(Rectangle::default()),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for IcedEditor<'a>
where
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _mouse_cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        self.last_bounds.set(bounds);
        let origin = Point::new(bounds.x, bounds.y);

        // ── Фаза 1: перешейп (mutable borrow shaped_doc) ──────────────
        if self.inner.dirty.get() {
            let content = self.inner.content.borrow();
            let cursor_line = self.inner.cursor.borrow().line();
            let scroll_y = self.inner.scroll_y.get();
            let mode = self.inner.mode;
            let theme = &self.inner.theme;
            let mut shaped = self.inner.shaped_doc.borrow_mut();
            render::build(
                &mut *shaped,
                &content,
                &self.inner.cache,
                mode,
                cursor_line,
                theme,
                self.inner.base_size,
                self.inner.heading_size,
                scroll_y,
                Some(bounds.height),
            );
            drop(shaped);
            self.inner.dirty.set(false);
        }

        // ── Фаза 2: отрисовка текста GPU (fill_text) ─────────────────
        // --- Фон ---
        let bg = &self.inner.theme.background;
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

        // --- Текст (по layout_run'ам) ---
        let shaped = self.inner.shaped_doc.borrow();
        let scroll_y = self.inner.scroll_y.get();
        let text_color = self.inner.theme.text.color;
        let iced_color = Color::from_rgba8(
            (text_color.r * 255.0) as u8,
            (text_color.g * 255.0) as u8,
            (text_color.b * 255.0) as u8,
            text_color.a as f32,
        );
        for run in shaped.buffer.layout_runs() {
            let line_top = run.line_top - scroll_y;
            // Пропускаем строки, содержащие только zero-width space (пустые строки)
            if run.text == "\u{200B}" {
                continue;
            }

            renderer.fill_text(
                text::Text {
                    content: run.text.to_string(),
                    bounds: Size::new(f32::INFINITY, run.line_height),
                    size: Pixels(self.inner.base_size),
                    line_height: text::LineHeight::Absolute(Pixels(run.line_height)),
                    font: renderer.default_font(),
                    align_x: text::Alignment::Left,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                },
                Point::new(origin.x, origin.y + line_top),
                iced_color,
                Rectangle::new(origin, bounds.size()),
            );
        }

        // --- Курсор ---
        let cursor = self.inner.cursor.borrow();
        let content = self.inner.content.borrow();
        if self.inner.mode != EditMode::Preview && cursor.should_blink() {
            let (line_start, _) = cursor_line_bounds(&content, cursor.line());
            let byte_in_line = cursor.raw().saturating_sub(line_start);

            let mut cursor_x = 0.0;
            let mut cursor_y = 0.0;
            let mut line_h = 12.0;

            for run in shaped.buffer.layout_runs() {
                if run.line_i != cursor.line() {
                    continue;
                }
                cursor_y = run.line_top - scroll_y;
                line_h = run.line_height;

                let mut found = false;
                for glyph in run.glyphs.iter() {
                    if glyph.start >= byte_in_line {
                        cursor_x = glyph.x;
                        found = true;
                        break;
                    }
                }
                if !found {
                    cursor_x = run
                        .glyphs
                        .last()
                        .map(|g| g.x + g.w)
                        .unwrap_or(0.0);
                }
                break;
            }

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

    fn update(
        &mut self,
        _tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor_state: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let origin = Point::new(bounds.x, bounds.y);

        match event {
            Event::Keyboard(kb_event) => {
                if let iced::keyboard::Event::KeyPressed {
                    key,
                    physical_key,
                    modifiers,
                    text,
                    ..
                } = kb_event
                {
                    let cmd = modifiers.command();

                    // Ctrl+S
                    if cmd
                        && key
                            .to_latin(*physical_key)
                            .is_some_and(|c| c == 's')
                    {
                        let content = self.inner.content.borrow();
                        let path = &self.inner.file_path;
                        if let Err(e) = std::fs::write(path, content.as_bytes()) {
                            eprintln!("[Zol] Ошибка сохранения {path}: {e}");
                        } else {
                            eprintln!("[Zol] Сохранено в {path}");
                        }
                        return;
                    }

                    use iced::keyboard::key::Named;

                    match key.as_ref() {
                        iced::keyboard::Key::Named(Named::ArrowLeft) => {
                            let c = self.inner.content.borrow();
                            self.inner.cursor.borrow_mut().move_left(&c);
                        }
                        iced::keyboard::Key::Named(Named::ArrowRight) => {
                            let c = self.inner.content.borrow();
                            self.inner.cursor.borrow_mut().move_right(&c);
                        }
                        iced::keyboard::Key::Named(Named::ArrowUp) => {
                            let target = {
                                let c = self.inner.content.borrow();
                                let cl = self.inner.cursor.borrow().line();
                                let n = c.bytes().filter(|&b| b == b'\n').count() + 1;
                                if cl > 0 { Some(cl - 1) } else { None }
                                    .filter(|&t| t < n)
                            };
                            if let Some(t) = target {
                                super::nav::move_vertical(self.inner, t);
                            }
                        }
                        iced::keyboard::Key::Named(Named::ArrowDown) => {
                            let target = {
                                let c = self.inner.content.borrow();
                                let cl = self.inner.cursor.borrow().line();
                                let n = c.bytes().filter(|&b| b == b'\n').count() + 1;
                                if cl + 1 < n { Some(cl + 1) } else { None }
                            };
                            if let Some(t) = target {
                                super::nav::move_vertical(self.inner, t);
                            }
                        }
                        iced::keyboard::Key::Named(Named::Home) => {
                            let c = self.inner.content.borrow();
                            self.inner.cursor.borrow_mut().move_home(&c);
                        }
                        iced::keyboard::Key::Named(Named::End) => {
                            let c = self.inner.content.borrow();
                            self.inner.cursor.borrow_mut().move_end(&c);
                        }
                        iced::keyboard::Key::Named(Named::Backspace) => {
                            let raw = self.inner.cursor.borrow().raw();
                            let content = self.inner.content.borrow();
                            let prev = if raw > 0 && !content.is_empty() {
                                cursor::prev_grapheme_boundary(&content, raw).unwrap_or(0)
                            } else {
                                raw
                            };
                            if prev != raw {
                                drop(content);
                                self.inner.content.borrow_mut().drain(prev..raw);
                                let c = self.inner.content.borrow();
                                self.inner.cursor.borrow_mut().set_raw(&c, prev);
                                self.inner.dirty.set(true);
                            }
                        }
                        iced::keyboard::Key::Named(Named::Delete) => {
                            let raw = self.inner.cursor.borrow().raw();
                            let content = self.inner.content.borrow();
                            let next = if raw < content.len() && !content.is_empty() {
                                cursor::next_grapheme_boundary(&content, raw)
                                    .unwrap_or(content.len())
                            } else {
                                raw
                            };
                            if next != raw {
                                drop(content);
                                self.inner.content.borrow_mut().drain(raw..next);
                                let c = self.inner.content.borrow();
                                self.inner.cursor.borrow_mut().set_raw(&c, raw);
                                self.inner.dirty.set(true);
                            }
                        }
                        iced::keyboard::Key::Named(Named::Enter) => {
                            let raw = self.inner.cursor.borrow().raw();
                            self.inner.content.borrow_mut().insert(raw, '\n');
                            let c = self.inner.content.borrow();
                            self.inner.cursor.borrow_mut().set_raw(&c, raw + 1);
                            self.inner.cursor.borrow_mut().reset_col_visual();
                            self.inner.dirty.set(true);
                        }
                        _ => {
                            if let Some(text) = text {
                                if !cmd && !modifiers.alt() {
                                    let mut raw = self.inner.cursor.borrow().raw();
                                    for ch in text.chars() {
                                        if !ch.is_control() {
                                            self.inner.content.borrow_mut().insert(raw, ch);
                                            raw += ch.len_utf8();
                                        }
                                    }
                                    let c = self.inner.content.borrow();
                                    self.inner.cursor.borrow_mut().set_raw(&c, raw);
                                    self.inner.dirty.set(true);
                                }
                            }
                        }
                    }

                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse_event) => {
                if let iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) = mouse_event {
                    if let Some(pos) = cursor_state.position_in(bounds) {
                        let local_x = pos.x - origin.x;
                        let local_y = pos.y - origin.y;

                        let shaped = self.inner.shaped_doc.borrow();
                        let scroll_y = self.inner.scroll_y.get();
                        let cosmic_cursor = shaped.buffer.hit(local_x, local_y + scroll_y);

                        if let Some(cosmic) = cosmic_cursor {
                            let content = self.inner.content.borrow();
                            let (line_start, _) =
                                cursor_line_bounds(&content, cosmic.line);
                            let new_raw =
                                (line_start + cosmic.index).min(content.len());
                            drop(content);
                            let c = self.inner.content.borrow();
                            let mut cursor = self.inner.cursor.borrow_mut();
                            cursor.set_raw(&c, new_raw);
                            cursor.set_line(cosmic.line);
                            cursor.reset_col_visual();
                            shell.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Text
    }
}

/// Создать `Element` с редактором.
pub fn editor_element<'a, Message: 'a>(
    inner: &'a EditorInner,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    Element::new(IcedEditor::new(inner))
}
