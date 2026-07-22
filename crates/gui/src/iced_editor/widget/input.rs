//! Обработка событий ввода (клавиатура, мышь).
//!
//! Все мутации документа — через [`EditorInner::edit_doc()`].
//! Автоскролл вынесен в отдельную функцию `auto_scroll`.

use iced::advanced::{mouse, Shell};
use iced::{Event, Point, Rectangle};
use iced::mouse::ScrollDelta;
use iced::advanced::Layout;
use iced::keyboard::{self, key::Named};

use api::cursor as api_cursor;
use api::file as api_file;
use api::text as api_text;
use editor::layout::cursor_line_bounds;

use super::editor::IcedEditor;

/// Точка входа для `Widget::update()`.
pub fn update<'a, Message>(
    this: &mut IcedEditor<'a>,
    event: &Event,
    layout: Layout<'_>,
    cursor_state: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let bounds = layout.bounds();
    let origin = Point::new(bounds.x, bounds.y);

    match event {
        Event::Keyboard(kb_event) => {
            handle_keyboard(this, kb_event, bounds, origin, shell);
        }
        Event::Mouse(mouse_event) => {
            handle_mouse(this, mouse_event, bounds, origin, cursor_state, shell);
        }
        _ => {}
    }
}

// ─────────────────────────────────────────────────────────────────────
// Клавиатура
// ─────────────────────────────────────────────────────────────────────

fn handle_keyboard<'a, Message>(
    this: &mut IcedEditor<'a>,
    kb_event: &keyboard::Event,
    bounds: Rectangle,
    _origin: Point,
    shell: &mut Shell<'_, Message>,
) {
    let keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        text,
        ..
    } = kb_event
    else {
        return;
    };

    let cmd = modifiers.command();

    // Ctrl+S — сохранить файл
    if cmd && key.to_latin(*physical_key).is_some_and(|c| c == 's') {
        let doc = this.inner.doc.borrow();
        if let Err(e) = api_file::file_save(&doc, &this.inner.file_path) {
            eprintln!("[Zol] Ошибка сохранения {}: {}", this.inner.file_path, e);
        } else {
            eprintln!("[Zol] Сохранено в {}", this.inner.file_path);
        }
        shell.request_redraw();
        return;
    }

    match key.as_ref() {
        // ── Навигация ──
        keyboard::Key::Named(Named::ArrowLeft) => {
            let mut doc = this.inner.doc.borrow_mut();
            api_cursor::move_left(&mut *doc);
        }
        keyboard::Key::Named(Named::ArrowRight) => {
            let mut doc = this.inner.doc.borrow_mut();
            api_cursor::move_right(&mut *doc);
        }
        keyboard::Key::Named(Named::ArrowUp) => {
            let mut doc = this.inner.doc.borrow_mut();
            api_cursor::move_up(&mut *doc);
        }
        keyboard::Key::Named(Named::ArrowDown) => {
            let mut doc = this.inner.doc.borrow_mut();
            api_cursor::move_down(&mut *doc);
        }
        keyboard::Key::Named(Named::Home) => {
            let mut doc = this.inner.doc.borrow_mut();
            api_cursor::move_home(&mut *doc);
        }
        keyboard::Key::Named(Named::End) => {
            let mut doc = this.inner.doc.borrow_mut();
            api_cursor::move_end(&mut *doc);
        }

        // ── Tab: циклическое переключение режимов ──
        keyboard::Key::Named(Named::Tab) => {
            this.inner.cycle_mode();
        }

        // ── Редактирование ──
        keyboard::Key::Named(Named::Backspace) => {
            this.inner.edit_doc(|doc| {
                api_text::delete_before(doc);
            });
        }
        keyboard::Key::Named(Named::Delete) => {
            this.inner.edit_doc(|doc| {
                api_text::delete_after(doc);
            });
        }
        keyboard::Key::Named(Named::Enter) => {
            this.inner.edit_doc(|doc| {
                api_text::newline(doc);
            });
        }
        _ => {
            // Обычный ввод символов (не под модификаторами)
            if let Some(text) = text {
                if !cmd && !modifiers.alt() {
                    let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !filtered.is_empty() {
                        this.inner.edit_doc(|doc| {
                            api_text::insert_at_cursor(doc, &filtered);
                        });
                    }
                }
            }
        }
    }

    // Автоскролл после навигации/редактирования
    auto_scroll(this, bounds);
    shell.request_redraw();
}

// ─────────────────────────────────────────────────────────────────────
// Мышь
// ─────────────────────────────────────────────────────────────────────

fn handle_mouse<'a, Message>(
    this: &mut IcedEditor<'a>,
    mouse_event: &iced::mouse::Event,
    bounds: Rectangle,
    origin: Point,
    cursor_state: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    match mouse_event {
        iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
            if let Some(pos) = cursor_state.position_in(bounds) {
                let local_x = pos.x - origin.x;
                let local_y = pos.y - origin.y;

                let shaped = this.inner.shaped_doc.borrow();
                let scroll_y = this.inner.scroll_y.get();
                let cosmic_cursor = shaped.buffer.hit(local_x, local_y + scroll_y);

                if let Some(cosmic) = cosmic_cursor {
                    let content = this.inner.doc.borrow().content.clone();
                    let (line_start, _) = cursor_line_bounds(&content, cosmic.line);
                    let new_raw = (line_start + cosmic.index).min(content.len());

                    let mut doc = this.inner.doc.borrow_mut();
                    api_cursor::cursor_set_raw(&mut *doc, new_raw);
                    api_cursor::cursor_set_line(&mut *doc, cosmic.line);
                    api_cursor::cursor_reset_col(&mut *doc);
                }
            }

            auto_scroll(this, bounds);
            shell.request_redraw();
        }
        iced::mouse::Event::WheelScrolled { delta } => {
            let amount = match delta {
                ScrollDelta::Lines { y, .. } => -y * 40.0,
                ScrollDelta::Pixels { y, .. } => -y,
            };
            if amount.abs() > 0.0 {
                let max_scroll =
                    (this.inner.shaped_doc.borrow().total_height() - bounds.height).max(0.0);
                let new_scroll =
                    (this.inner.scroll_y.get() + amount).clamp(0.0, max_scroll);
                this.inner.scroll_y.set(new_scroll);
                this.inner.mark_dirty();
                shell.request_redraw();
            }
        }
        _ => {}
    }
}

// ─────────────────────────────────────────────────────────────────────
// Автоскролл (вынесен, чтобы не дублировать)
// ─────────────────────────────────────────────────────────────────────

/// Проверить, виден ли курсор, и если нет — скорректировать `scroll_y`.
fn auto_scroll(this: &IcedEditor<'_>, bounds: Rectangle) {
    let cursor_line = this.inner.doc.borrow().cursor.line();
    let new_scroll_y = super::super::scroll::ensure_cursor_visible(
        this.inner.scroll_y.get(),
        bounds.height,
        &this.inner.shaped_doc.borrow(),
        cursor_line,
    );
    if (new_scroll_y - this.inner.scroll_y.get()).abs() > 0.5 {
        this.inner.scroll_y.set(new_scroll_y);
        this.inner.mark_dirty();
    }
}
