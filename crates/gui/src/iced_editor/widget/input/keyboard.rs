//! Обработка событий клавиатуры.

use iced::advanced::Shell;
use iced::keyboard::{self, key::Named};
use iced::{Rectangle, Point};

use api::cursor as api_cursor;
use api::file as api_file;

use super::IcedEditor;
use super::auto_scroll;

pub fn handle_keyboard<'a, Message>(
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

        keyboard::Key::Named(Named::Tab) => {
            this.inner.cycle_mode();
        }

        keyboard::Key::Named(Named::Backspace) => {
            let (from, to) = {
                let doc = this.inner.doc.borrow();
                let raw = doc.cursor.raw();
                if raw == 0 || doc.content().is_empty() {
                    (0, 0)
                } else {
                    let prev = editor::cursor::prev_grapheme_boundary(doc.content(), raw).unwrap_or(0);
                    (prev, raw)
                }
            };
            if from < to {
                this.inner.edit_doc_raw(from, to, "");
                let mut doc = this.inner.doc.borrow_mut();
                doc.set_cursor_raw(from);
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
            let mut doc = this.inner.doc.borrow_mut();
            doc.set_cursor_raw(raw + 1);
            doc.cursor.reset_col_visual();
        }
        _ => {
            if let Some(text) = text {
                if !cmd && !modifiers.alt() {
                    let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !filtered.is_empty() {
                        let raw = this.inner.doc.borrow().cursor.raw();
                        this.inner.edit_doc_raw(raw, raw, &filtered);
                        let mut doc = this.inner.doc.borrow_mut();
                        doc.set_cursor_raw(raw + filtered.len());
                    }
                }
            }
        }
    }

    auto_scroll(this, bounds);
    shell.request_redraw();
}
