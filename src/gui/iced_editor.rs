//! Iced-виджет редактора — рисует через `fill_quad` (без cosmic-text fill_raw).
//!
//! Внутреннее состояние (`EditorInner`) хранится в `AppState` за `RefCell`,
//! виджет заимствует его через `&RefCell`. Аналог `text_editor::Content`.
//!
//! TODO:
//!   - Использовать `fill_raw` после стабилизации Arc<Buffer> в ShapedDocument
//!   - Выделение (selection) через fill_quad с фоном
//!   - Скроллинг

use std::cell::{Ref, RefCell, RefMut};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::{
    Background, Color, Element, Event, Length, Point, Rectangle, Size,
};

use crate::editor::cursor::{self, Cursor};
use crate::editor::layout::cursor_line_bounds;
use crate::editor::render::ShapedDocument;
use crate::editor::state::EditMode;

// ---------------------------------------------------------------------------
// Внутреннее состояние
// ---------------------------------------------------------------------------

/// Состояние редактора, спрятанное за `RefCell`.
pub struct EditorInner {
    pub content: RefCell<String>,
    pub cursor: RefCell<Cursor>,
    pub shaped_doc: ShapedDocument,
    pub mode: EditMode,
}

impl EditorInner {
    pub fn new(content: String, shaped_doc: ShapedDocument) -> Self {
        Self {
            content: RefCell::new(content),
            cursor: RefCell::new(Cursor::new()),
            shaped_doc,
            mode: EditMode::LivePreview,
        }
    }
}

// ---------------------------------------------------------------------------
// Виджет (заимствует RefCell)
// ---------------------------------------------------------------------------

/// Iced-виджет редактора — заимствует `EditorInner` через `&RefCell`.
pub struct IcedEditor<'a> {
    inner: &'a RefCell<EditorInner>,
}

impl<'a> IcedEditor<'a> {
    pub fn new(inner: &'a RefCell<EditorInner>) -> Self {
        Self { inner }
    }

    fn borrow(&self) -> Ref<'_, EditorInner> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, EditorInner> {
        self.inner.borrow_mut()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for IcedEditor<'a>
where
    Renderer: iced::advanced::Renderer,
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
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let inner = self.borrow();
        let bounds = layout.bounds();
        let origin = Point::new(bounds.x, bounds.y);

        // --- Фон ---
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            Background::Color(Color::from_rgba8(30, 30, 30, 1.0)),
        );

        // --- Глифы ---
        for run in inner.shaped_doc.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let x = origin.x + glyph.x;
                let y = origin.y + run.line_y + glyph.y;
                let w = glyph.w;
                let h = glyph.font_size;

                let color: Color = glyph.color_opt.map_or(Color::WHITE, |c| {
                    Color::from_rgba8(c.r(), c.g(), c.b(), c.a() as f32 / 255.0)
                });

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(x, y),
                            Size::new(w, h),
                        ),
                        ..renderer::Quad::default()
                    },
                    color,
                );
            }
        }

        // --- Курсор ---
        let cursor = inner.cursor.borrow();
        let content = inner.content.borrow();
        if inner.mode != EditMode::Preview && cursor.should_blink() {
            let (line_start, _) =
                cursor_line_bounds(&content, cursor.line());
            let byte_in_line =
                cursor.raw().saturating_sub(line_start);

            let mut cursor_x = 0.0;
            let mut cursor_y = 0.0;
            let mut line_h = 12.0;

            for run in inner.shaped_doc.buffer.layout_runs() {
                if run.line_i != cursor.line() {
                    continue;
                }
                cursor_y = run.line_top;
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
                    cursor_x = run.glyphs.last().map(|g| g.x + g.w).unwrap_or(0.0);
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
                        eprintln!("[Flint] Сохранение не подключено в Iced");
                        return;
                    }

                    use iced::keyboard::key::Named;

                    let editor = self.inner.borrow();
                    match key.as_ref() {
                        iced::keyboard::Key::Named(Named::ArrowLeft) => {
                            let mut cursor = editor.cursor.borrow_mut();
                            let c = editor.content.borrow();
                            cursor.move_left(&c);
                        }
                        iced::keyboard::Key::Named(Named::ArrowRight) => {
                            let mut cursor = editor.cursor.borrow_mut();
                            let c = editor.content.borrow();
                            cursor.move_right(&c);
                        }
                        iced::keyboard::Key::Named(Named::ArrowUp) => {
                            // TODO: move_up
                        }
                        iced::keyboard::Key::Named(Named::ArrowDown) => {
                            // TODO: move_down
                        }
                        iced::keyboard::Key::Named(Named::Home) => {
                            let mut cursor = editor.cursor.borrow_mut();
                            let c = editor.content.borrow();
                            cursor.move_home(&c);
                        }
                        iced::keyboard::Key::Named(Named::End) => {
                            let mut cursor = editor.cursor.borrow_mut();
                            let c = editor.content.borrow();
                            cursor.move_end(&c);
                        }
                        iced::keyboard::Key::Named(Named::Backspace) => {
                            let raw;
                            let prev;
                            {
                                let cursor = editor.cursor.borrow();
                                let c = editor.content.borrow();
                                raw = cursor.raw();
                                prev = if raw > 0 && !c.is_empty() {
                                    cursor::prev_grapheme_boundary(&c, raw).unwrap_or(0)
                                } else {
                                    raw
                                };
                            }
                            if prev != raw {
                                let mut c = editor.content.borrow_mut();
                                c.drain(prev..raw);
                                drop(c);
                                let mut cursor = editor.cursor.borrow_mut();
                                let c = editor.content.borrow();
                                cursor.set_raw(&c, prev);
                            }
                        }
                        iced::keyboard::Key::Named(Named::Delete) => {
                            let raw;
                            let next;
                            {
                                let cursor = editor.cursor.borrow();
                                let c = editor.content.borrow();
                                raw = cursor.raw();
                                next = if raw < c.len() && !c.is_empty() {
                                    cursor::next_grapheme_boundary(&c, raw).unwrap_or(c.len())
                                } else {
                                    raw
                                };
                            }
                            if next != raw {
                                let mut c = editor.content.borrow_mut();
                                c.drain(raw..next);
                                drop(c);
                                let mut cursor = editor.cursor.borrow_mut();
                                let c = editor.content.borrow();
                                cursor.set_raw(&c, raw);
                            }
                        }
                        iced::keyboard::Key::Named(Named::Enter) => {
                            let raw;
                            {
                                let cursor = editor.cursor.borrow();
                                raw = cursor.raw();
                            }
                            {
                                let mut c = editor.content.borrow_mut();
                                c.insert(raw, '\n');
                            }
                            {
                                let mut cursor = editor.cursor.borrow_mut();
                                let c = editor.content.borrow();
                                cursor.set_raw(&c, raw + 1);
                                cursor.reset_col_visual();
                            }
                        }
                        _ => {
                            if let Some(text) = text {
                                if !cmd && !modifiers.alt() {
                                    let mut raw;
                                    {
                                        let cursor = editor.cursor.borrow();
                                        raw = cursor.raw();
                                    }
                                    for ch in text.chars() {
                                        if !ch.is_control() {
                                            let mut c = editor.content.borrow_mut();
                                            c.insert(raw, ch);
                                            raw += ch.len_utf8();
                                        }
                                    }
                                    {
                                        let mut cursor = editor.cursor.borrow_mut();
                                        let c = editor.content.borrow();
                                        cursor.set_raw(&c, raw);
                                    }
                                }
                            }
                        }
                    }

                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse_event) => {
                if let iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Left,
                ) = mouse_event
                {
                    if let Some(pos) = cursor_state.position_in(bounds) {
                        let local_x = pos.x - origin.x;
                        let local_y = pos.y - origin.y;

                        let editor = self.inner.borrow();
                        let cosmic_cursor =
                            editor.shaped_doc.buffer.hit(local_x, local_y);

                        if let Some(cosmic) = cosmic_cursor {
                            let content = editor.content.borrow();
                            let (line_start, _) = cursor_line_bounds(
                                &content,
                                cosmic.line,
                            );
                            let new_raw =
                                (line_start + cosmic.index).min(content.len());
                            drop(content);
                            let mut cursor = editor.cursor.borrow_mut();
                            let c = editor.content.borrow();
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

// ---------------------------------------------------------------------------
// Хелперы (копия api/text, но без EditorWidget)
// ---------------------------------------------------------------------------

fn delete_before(content: &mut String, cursor: &mut Cursor) {
    let raw = cursor.raw();
    if raw == 0 || content.is_empty() {
        return;
    }
    let prev = cursor::prev_grapheme_boundary(content, raw).unwrap_or(0);
    content.drain(prev..raw);
    cursor.set_raw(content, prev);
}

fn delete_after(content: &mut String, cursor: &mut Cursor) {
    let raw = cursor.raw();
    if raw >= content.len() || content.is_empty() {
        return;
    }
    let next =
        cursor::next_grapheme_boundary(content, raw).unwrap_or(content.len());
    content.drain(raw..next);
    cursor.set_raw(content, raw);
}

/// Создать `Element` с редактором.
pub fn editor_element<'a, Message: 'a>(
    inner: &'a RefCell<EditorInner>,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    Element::new(IcedEditor::new(inner))
}
