//! Iced-виджет редактора — рисует через `fill_quad` (без cosmic-text fill_raw).
//!
//! Внутреннее состояние (`EditorInner`) хранится в `AppState` за `RefCell`,
//! виджет заимствует его через `&RefCell`. Аналог `text_editor::Content`.
//!
//! TODO:
//!   - Использовать `fill_raw` после стабилизации Arc<Buffer> в ShapedDocument
//!   - Выделение (selection) через fill_quad с фоном
//!   - Скроллинг

use std::cell::{Cell, RefCell};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::{
    Background, Color, Element, Event, Length, Point, Rectangle, Size,
};

use crate::editor::cache::DocumentCache;
use crate::editor::cursor::{self, Cursor};
use crate::editor::layout::cursor_line_bounds;
use crate::editor::render::{self, ShapedDocument};
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;

// ---------------------------------------------------------------------------
// Внутреннее состояние
// ---------------------------------------------------------------------------

/// Состояние редактора.
///
/// Поля-`RefCell` обеспечивают interior mutability — виджет держит
/// `&EditorInner`, а мутации происходят через `.borrow_mut()` отдельных
/// полей. `dirty` сигнализирует `draw()`, что `shaped_doc` нужно
/// перестроить.
pub struct EditorInner {
    pub content: RefCell<String>,
    pub cursor: RefCell<Cursor>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: DocumentCache,
    pub mode: EditMode,
    pub dirty: Cell<bool>,
    pub base_size: f32,
    pub heading_size: f32,
    pub theme: EditorTheme,
}

impl EditorInner {
    pub fn new(content: String, shaped_doc: ShapedDocument) -> Self {
        Self {
            content: RefCell::new(content),
            cursor: RefCell::new(Cursor::new()),
            shaped_doc: RefCell::new(shaped_doc),
            cache: DocumentCache::default(),
            mode: EditMode::LivePreview,
            dirty: Cell::new(false),
            base_size: 14.0,
            heading_size: 24.0,
            theme: EditorTheme::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Виджет (держит &EditorInner — interior mutability через RefCell-поля)
// ---------------------------------------------------------------------------

/// Iced-виджет редактора.
pub struct IcedEditor<'a> {
    inner: &'a EditorInner,
}

impl<'a> IcedEditor<'a> {
    pub fn new(inner: &'a EditorInner) -> Self {
        Self { inner }
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
        let bounds = layout.bounds();
        let origin = Point::new(bounds.x, bounds.y);

        // ── Фаза 1: перешейп (mutable borrow shaped_doc) ──────────────
        if self.inner.dirty.get() {
            let content = self.inner.content.borrow();
            let cache = &self.inner.cache;
            let cursor_line = self.inner.cursor.borrow().line();
            let mode = self.inner.mode;
            let theme = &self.inner.theme;
            let mut shaped = self.inner.shaped_doc.borrow_mut();
            render::build(
                &mut *shaped,
                &content,
                cache,
                mode,
                cursor_line,
                theme,
                self.inner.base_size,
                self.inner.heading_size,
                Some(bounds.height),
            );
            // shaped, content, cursor_line — dropp'ятся здесь
            // self.inner.dirty = false; — сделаем ниже в отдельном блоге
        }

        // Сбрасываем dirty после перешейпа (отдельный блок, чтобы не
        // пересекаться с borrow_mut shaped_doc).
        if self.inner.dirty.get() {
            self.inner.dirty.set(false);
        }

        // ── Фаза 2: отрисовка ──────────────────────────────────────────

        // --- Фон ---
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            Background::Color(Color::from_rgba8(30, 30, 30, 1.0)),
        );

        // --- Глифы ---
        let shaped = self.inner.shaped_doc.borrow();
        for run in shaped.buffer.layout_runs() {
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
        let cursor = self.inner.cursor.borrow();
        let content = self.inner.content.borrow();
        if self.inner.mode != EditMode::Preview && cursor.should_blink() {
            let (line_start, _) =
                cursor_line_bounds(&content, cursor.line());
            let byte_in_line =
                cursor.raw().saturating_sub(line_start);

            let mut cursor_x = 0.0;
            let mut cursor_y = 0.0;
            let mut line_h = 12.0;

            // shaped уже borrow'нуто выше — используем его
            for run in shaped.buffer.layout_runs() {
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
                        eprintln!("[Zol] Сохранение не подключено в Iced");
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
                                let n =
                                    c.bytes().filter(|&b| b == b'\n').count() + 1;
                                if cl > 0 {
                                    Some(cl - 1)
                                } else {
                                    None
                                }
                                .filter(|&t| t < n)
                            };
                            if let Some(t) = target {
                                move_vertical(self.inner, t);
                            }
                        }
                        iced::keyboard::Key::Named(Named::ArrowDown) => {
                            let target = {
                                let c = self.inner.content.borrow();
                                let cl = self.inner.cursor.borrow().line();
                                let n =
                                    c.bytes().filter(|&b| b == b'\n').count() + 1;
                                if cl + 1 < n {
                                    Some(cl + 1)
                                } else {
                                    None
                                }
                            };
                            if let Some(t) = target {
                                move_vertical(self.inner, t);
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
                                cursor::next_grapheme_boundary(&content, raw).unwrap_or(content.len())
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
                if let iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Left,
                ) = mouse_event
                {
                    if let Some(pos) = cursor_state.position_in(bounds) {
                        let local_x = pos.x - origin.x;
                        let local_y = pos.y - origin.y;

                        let shaped = self.inner.shaped_doc.borrow();
                        let cosmic_cursor =
                            shaped.buffer.hit(local_x, local_y);

                        if let Some(cosmic) = cosmic_cursor {
                            let content = self.inner.content.borrow();
                            let (line_start, _) = cursor_line_bounds(
                                &content,
                                cosmic.line,
                            );
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

// ---------------------------------------------------------------------------
// Вертикальная навигация (сохранение пиксельной X, как в painter.rs)
// ---------------------------------------------------------------------------

/// X-позиция курсора на строке `line` по глифам буфера.
fn cursor_x_on_line(shaped: &ShapedDocument, line: usize, byte_in_line: usize) -> f32 {
    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            if glyph.start >= byte_in_line {
                return glyph.x;
            }
        }
        return run.glyphs.last().map(|g| g.x + g.w).unwrap_or(0.0);
    }
    0.0
}

/// Ближайший к `x` content-offset на строке `line`.
///
/// Пустая строка → начало строки. Иначе среди глифов и конца строки
/// выбирается точка с минимальным расстоянием по X.
fn raw_at_x_on_line(
    shaped: &ShapedDocument,
    line: usize,
    x: f32,
    line_start: usize,
    line_end: usize,
) -> usize {
    if line_end <= line_start {
        return line_start;
    }
    let mut best: Option<(f32, usize)> = None;
    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            let dist = (glyph.x - x).abs();
            let cand = line_start + glyph.start;
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, cand));
            }
        }
        if let Some(last) = run.glyphs.last() {
            let end_x = last.x + last.w;
            let dist = (end_x - x).abs();
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, line_end));
            }
        }
        break;
    }
    best.map_or(line_start, |(_, c)| c)
}

/// Переместить курсор на строку `target_line`, сохраняя пиксельную X.
fn move_vertical(inner: &EditorInner, target_line: usize) {
    let x = {
        let content = inner.content.borrow();
        let shaped = inner.shaped_doc.borrow();
        let cursor = inner.cursor.borrow();
        let cl = cursor.line();
        let (ls, _) = cursor_line_bounds(&content, cl);
        let byte_in_line = cursor.raw().saturating_sub(ls);
        cursor_x_on_line(&shaped, cl, byte_in_line)
    };

    let new_raw = {
        let content = inner.content.borrow();
        let shaped = inner.shaped_doc.borrow();
        let (t_start, t_end) = cursor_line_bounds(&content, target_line);
        raw_at_x_on_line(&shaped, target_line, x, t_start, t_end)
    };

    let c = inner.content.borrow();
    let mut cursor = inner.cursor.borrow_mut();
    cursor.set_raw(&c, new_raw);
    cursor.set_col_visual(x);
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
    inner: &'a EditorInner,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    Element::new(IcedEditor::new(inner))
}
