//! Обработка событий клавиатуры.

use iced::Rectangle;
use iced::advanced::Shell;
use iced::keyboard::{self, key::Named};

use api::cursor as api_cursor;
use api::file as api_file;
use editor::state::EditMode;

use super::auto_scroll;
use crate::iced_editor::widget::editor::IcedEditor;

/// Переместить курсор и пометить документ как грязный для перешейпа.
fn move_cursor(this: &mut IcedEditor<'_>, f: impl FnOnce(&mut editor::document::Document)) {
    let mut doc = this.inner.doc.borrow_mut();
    f(&mut *doc);
    drop(doc);
    this.inner.mark_dirty();
}

pub fn handle_keyboard<'a, Message>(
    this: &mut IcedEditor<'a>,
    kb_event: &keyboard::Event,
    bounds: Rectangle,
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

    // ─── Ctrl+ ... ─────────────────────────────────────────────────
    if cmd {
        // Ctrl+S — сохранить файл
        if key.to_latin(*physical_key).is_some_and(|c| c == 's') {
            let doc = this.inner.doc.borrow();
            if let Err(e) = api_file::file_save(&doc, &this.inner.file_path) {
                eprintln!("[Zol] Ошибка сохранения {}: {}", this.inner.file_path, e);
            } else {
                eprintln!("[Zol] Сохранено в {}", this.inner.file_path);
            }
            shell.request_redraw();
            return;
        }

        // Ctrl+Home — в начало документа
        if matches!(key.as_ref(), keyboard::Key::Named(Named::Home)) {
            move_cursor(this, |d| d.set_cursor_raw(0));
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }
        // Ctrl+End — в конец документа
        if matches!(key.as_ref(), keyboard::Key::Named(Named::End)) {
            let len = this.inner.doc.borrow().content().len();
            move_cursor(this, |d| d.set_cursor_raw(len));
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }

        // Ctrl+← — слово влево
        if matches!(key.as_ref(), keyboard::Key::Named(Named::ArrowLeft)) {
            move_cursor(this, api_cursor::move_word_left);
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }
        // Ctrl+→ — слово вправо
        if matches!(key.as_ref(), keyboard::Key::Named(Named::ArrowRight)) {
            move_cursor(this, api_cursor::move_word_right);
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }
    }

    // ─── Preview: только переключение режимов ──────────────────────
    if !this.inner.get_mode().is_editable() {
        match key.as_ref() {
            keyboard::Key::Named(Named::Tab) => {
                this.inner.cycle_mode();
                shell.request_redraw();
            }
            keyboard::Key::Named(Named::Escape) => {
                this.inner.set_mode(EditMode::Preview);
                shell.request_redraw();
            }
            _ => {}
        }
        return;
    }

    // ─── Обычные клавиши (без Ctrl) ────────────────────────────────
    match key.as_ref() {
        keyboard::Key::Named(Named::ArrowLeft) => move_cursor(this, api_cursor::move_left),
        keyboard::Key::Named(Named::ArrowRight) => move_cursor(this, api_cursor::move_right),
        keyboard::Key::Named(Named::ArrowUp) => move_cursor(this, api_cursor::move_up),
        keyboard::Key::Named(Named::ArrowDown) => move_cursor(this, api_cursor::move_down),
        keyboard::Key::Named(Named::Home) => move_cursor(this, api_cursor::move_home),
        keyboard::Key::Named(Named::End) => move_cursor(this, api_cursor::move_end),

        keyboard::Key::Named(Named::Tab) => {
            this.inner.cycle_mode();
            shell.request_redraw();
            return;
        }

        keyboard::Key::Named(Named::Escape) => {
            this.inner.set_mode(EditMode::Preview);
            shell.request_redraw();
            return;
        }

        keyboard::Key::Named(Named::PageUp) => {
            let line_h = this.inner.base_size * 1.4;
            let n = (bounds.height / line_h) as usize;
            let mut doc = this.inner.doc.borrow_mut();
            for _ in 0..n {
                doc.cursor_move_up();
            }
            drop(doc);
            this.inner.mark_dirty();
        }
        keyboard::Key::Named(Named::PageDown) => {
            let line_h = this.inner.base_size * 1.4;
            let n = (bounds.height / line_h) as usize;
            let mut doc = this.inner.doc.borrow_mut();
            for _ in 0..n {
                doc.cursor_move_down();
            }
            drop(doc);
            this.inner.mark_dirty();
        }

        keyboard::Key::Named(Named::Backspace) => {
            let (from, to) = {
                let doc = this.inner.doc.borrow();
                let raw = doc.cursor.raw();
                if raw == 0 || doc.content().is_empty() {
                    (0, 0)
                } else {
                    let prev =
                        editor::cursor::prev_grapheme_boundary(doc.content(), raw).unwrap_or(0);
                    (prev, raw)
                }
            };
            if from < to {
                this.inner.edit_doc_raw(from, to, "");
                move_cursor(this, |d| d.set_cursor_raw(from));
            }
        }
        keyboard::Key::Named(Named::Delete) => {
            let (from, to) = {
                let doc = this.inner.doc.borrow();
                let raw = doc.cursor.raw();
                if raw >= doc.content().len() || doc.content().is_empty() {
                    (0, 0)
                } else {
                    let next = editor::cursor::next_grapheme_boundary(doc.content(), raw)
                        .unwrap_or(doc.content().len());
                    (raw, next)
                }
            };
            if from < to {
                this.inner.edit_doc_raw(from, to, "");
            }
        }
        keyboard::Key::Named(Named::Enter) => {
            let raw = this.inner.doc.borrow().cursor.raw();
            this.inner.edit_doc_raw(raw, raw, "\n");
            move_cursor(this, |d| {
                d.set_cursor_raw(raw + 1);
                d.cursor.reset_col_visual();
            });
        }
        _ => {
            if let Some(text) = text {
                if !cmd && !modifiers.alt() {
                    let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !filtered.is_empty() {
                        let raw = this.inner.doc.borrow().cursor.raw();
                        this.inner.edit_doc_raw(raw, raw, &filtered);
                        move_cursor(this, |d| d.set_cursor_raw(raw + filtered.len()));
                    }
                }
            }
        }
    }

    auto_scroll(this, bounds);
    shell.request_redraw();
}
