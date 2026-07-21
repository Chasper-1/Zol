//! Iced-виджет редактора.
//!
//! Содержит [`IcedEditor`] — виджет, реализующий `Widget<Message, Theme, Renderer>`.
//! Рисует текст через GPU (`renderer.fill_text`), курсор — вертикальной полоской.
//! Позиционирование глифов и hit-testing — через cosmic-text.
//!
//! Все мутации документа — через [`EditorInner::edit_doc()`].
//!
//! TODO:
//!   - Выделение (selection) через fill_quad с фоном
//!   - Цветная разметка (bold, italic, code…)
//!   - Word wrap (buffer.set_size(Some(w), h))
//!   - Загрузка файла из аргументов командной строки
//!   - RON-конфиг для тем/шрифтов

use std::cell::Cell;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::advanced::text::{self};
use iced::{
    alignment, Color, Element, Event, Length, Pixels, Point, Rectangle, Size,
};
use iced::mouse::ScrollDelta;

use crate::api::cursor as api_cursor;
use crate::api::file as api_file;
use crate::api::text as api_text;
use crate::editor::layout::cursor_line_bounds;
use crate::editor::render;
use crate::editor::state::EditMode;

use super::inner::EditorInner;
use super::scroll::ensure_cursor_visible;

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
        // Забираем всё, что нужно из doc, до borrow_mut shaped_doc
        let needs_reshape = {
            let doc = self.inner.doc.borrow();
            doc.dirty
        };

        if needs_reshape {
            let (content, cursor_line, scroll_y) = {
                let doc = self.inner.doc.borrow();
                (doc.content.clone(), doc.cursor.line(), self.inner.scroll_y.get())
            };
            let mode = self.inner.mode;
            let theme = &self.inner.theme;
            let cache = self.inner.cache.borrow().clone();
            let mut shaped = self.inner.shaped_doc.borrow_mut();
            render::build(
                &mut *shaped,
                &content,
                &cache,
                mode,
                cursor_line,
                theme,
                self.inner.base_size,
                self.inner.heading_size,
                scroll_y,
                Some(bounds.height),
            );
            drop(shaped);
            self.inner.doc.borrow_mut().dirty = false;
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
        if self.inner.mode != EditMode::Preview {
            let (cursor_line, cursor_raw, should_blink) = {
                let doc = self.inner.doc.borrow();
                (doc.cursor.line(), doc.cursor.raw(), doc.cursor.should_blink())
            };

            if should_blink {
                let content = self.inner.doc.borrow().content.clone();
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

                        let mut found = false;
                        for glyph in run.glyphs.iter() {
                            if glyph.start >= byte_in_line {
                                cx = glyph.x;
                                found = true;
                                break;
                            }
                        }
                        if !found {
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

                    // Ctrl+S — сохранить файл через API-ручку
                    if cmd
                        && key
                            .to_latin(*physical_key)
                            .is_some_and(|c| c == 's')
                    {
                        let doc = self.inner.doc.borrow();
                        if let Err(e) = api_file::file_save(&doc, &self.inner.file_path) {
                            eprintln!("[Zol] Ошибка сохранения {}: {}", self.inner.file_path, e);
                        } else {
                            eprintln!("[Zol] Сохранено в {}", self.inner.file_path);
                        }
                        return;
                    }

                    use iced::keyboard::key::Named;

                    match key.as_ref() {
                        // ── Навигация (курсор без изменения контента) ──
                        iced::keyboard::Key::Named(Named::ArrowLeft) => {
                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::move_left(&mut *doc);
                        }
                        iced::keyboard::Key::Named(Named::ArrowRight) => {
                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::move_right(&mut *doc);
                        }
                        iced::keyboard::Key::Named(Named::ArrowUp) => {
                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::move_up(&mut *doc);
                        }
                        iced::keyboard::Key::Named(Named::ArrowDown) => {
                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::move_down(&mut *doc);
                        }
                        iced::keyboard::Key::Named(Named::Home) => {
                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::move_home(&mut *doc);
                        }
                        iced::keyboard::Key::Named(Named::End) => {
                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::move_end(&mut *doc);
                        }

                        // ── Редактирование (контент меняется) ──
                        iced::keyboard::Key::Named(Named::Backspace) => {
                            self.inner.edit_doc(|doc| {
                                api_text::delete_before(doc);
                            });
                        }
                        iced::keyboard::Key::Named(Named::Delete) => {
                            self.inner.edit_doc(|doc| {
                                api_text::delete_after(doc);
                            });
                        }
                        iced::keyboard::Key::Named(Named::Enter) => {
                            self.inner.edit_doc(|doc| {
                                api_text::newline(doc);
                            });
                        }
                        _ => {
                            if let Some(text) = text {
                                if !cmd && !modifiers.alt() {
                                    let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                                    if !filtered.is_empty() {
                                        self.inner.edit_doc(|doc| {
                                            api_text::insert_at_cursor(doc, &filtered);
                                        });
                                    }
                                }
                            }
                        }
                    }

                    // Автоскролл: проверяем, что курсор виден
                    {
                        let cursor_line = self.inner.doc.borrow().cursor.line();
                        let new_scroll_y = ensure_cursor_visible(
                            self.inner.scroll_y.get(),
                            bounds.height,
                            &self.inner.shaped_doc.borrow(),
                            cursor_line,
                        );
                        if (new_scroll_y - self.inner.scroll_y.get()).abs() > 0.5 {
                            self.inner.scroll_y.set(new_scroll_y);
                            self.inner.mark_dirty();
                        }
                    }
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    if let Some(pos) = cursor_state.position_in(bounds) {
                        let local_x = pos.x - origin.x;
                        let local_y = pos.y - origin.y;

                        let shaped = self.inner.shaped_doc.borrow();
                        let scroll_y = self.inner.scroll_y.get();
                        let cosmic_cursor = shaped.buffer.hit(local_x, local_y + scroll_y);

                        if let Some(cosmic) = cosmic_cursor {
                            let content = self.inner.doc.borrow().content.clone();
                            let (line_start, _) =
                                cursor_line_bounds(&content, cosmic.line);
                            let new_raw =
                                (line_start + cosmic.index).min(content.len());

                            let mut doc = self.inner.doc.borrow_mut();
                            api_cursor::cursor_set_raw(&mut *doc, new_raw);
                            api_cursor::cursor_set_line(&mut *doc, cosmic.line);
                            api_cursor::cursor_reset_col(&mut *doc);
                        }

                        // Автоскролл после клика
                        {
                            let cursor_line = self.inner.doc.borrow().cursor.line();
                            let new_scroll_y = ensure_cursor_visible(
                                self.inner.scroll_y.get(),
                                bounds.height,
                                &self.inner.shaped_doc.borrow(),
                                cursor_line,
                            );
                            if (new_scroll_y - self.inner.scroll_y.get()).abs() > 0.5 {
                                self.inner.scroll_y.set(new_scroll_y);
                                self.inner.mark_dirty();
                            }
                        }
                        shell.request_redraw();
                    }
                }
                iced::mouse::Event::WheelScrolled { delta } => {
                    let amount = match delta {
                        ScrollDelta::Lines { y, .. } => -y * 40.0,
                        ScrollDelta::Pixels { y, .. } => -y,
                    };
                    if amount.abs() > 0.0 {
                        let max_scroll =
                            (self.inner.shaped_doc.borrow().total_height() - bounds.height)
                                .max(0.0);
                        let new_scroll =
                            (self.inner.scroll_y.get() + amount).clamp(0.0, max_scroll);
                        self.inner.scroll_y.set(new_scroll);
                        self.inner.mark_dirty();
                        shell.request_redraw();
                    }
                }
                _ => {}
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
