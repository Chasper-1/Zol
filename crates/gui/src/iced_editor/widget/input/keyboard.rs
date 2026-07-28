//! Обработка событий клавиатуры.

use iced::Rectangle;
use iced::advanced::Shell;
use iced::keyboard::{self, key::Named};

use api::cursor as api_cursor;
use api::doc as api_doc;
use api::file as api_file;
use api::text as api_text;
use editor::state::EditMode;

use super::auto_scroll;
use crate::iced_editor::widget::editor::IcedEditor;

/// Применить замыкание к документу в `edit_doc` и запросить redraw.
fn edit(this: &mut IcedEditor<'_>, f: impl FnOnce(&mut editor::document::Document)) {
    this.inner.edit_doc(f);
}

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
    let shift = modifiers.shift();

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

        // Ctrl+A — выделить всё
        if key.to_latin(*physical_key).is_some_and(|c| c == 'a') {
            edit(this, api_doc::select_all);
            shell.request_redraw();
            return;
        }

        // Ctrl+Home — в начало документа
        if matches!(key.as_ref(), keyboard::Key::Named(Named::Home)) {
            move_cursor(this, api_cursor::move_to_document_start);
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }
        // Ctrl+End — в конец документа
        if matches!(key.as_ref(), keyboard::Key::Named(Named::End)) {
            move_cursor(this, api_cursor::move_to_document_end);
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }

        // Ctrl+← — слово влево
        if matches!(key.as_ref(), keyboard::Key::Named(Named::ArrowLeft)) {
            let f = if shift {
                api_cursor::move_word_left_select
            } else {
                api_cursor::move_word_left
            };
            move_cursor(this, f);
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }
        // Ctrl+→ — слово вправо
        if matches!(key.as_ref(), keyboard::Key::Named(Named::ArrowRight)) {
            let f = if shift {
                api_cursor::move_word_right_select
            } else {
                api_cursor::move_word_right
            };
            move_cursor(this, f);
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }

        // Ctrl+Backspace — удалить слово слева
        if matches!(key.as_ref(), keyboard::Key::Named(Named::Backspace)) {
            edit(
                this,
                if shift {
                    api_text::delete_line
                } else {
                    api_text::delete_word_before
                },
            );
            auto_scroll(this, bounds);
            shell.request_redraw();
            return;
        }
        // Ctrl+Delete — удалить слово справа / до конца строки
        if matches!(key.as_ref(), keyboard::Key::Named(Named::Delete)) {
            edit(
                this,
                if shift {
                    api_text::delete_to_line_end
                } else {
                    api_text::delete_word_after
                },
            );
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
        // Shift+движение — расширение выделения
        keyboard::Key::Named(Named::ArrowLeft) if shift => {
            move_cursor(this, api_cursor::move_left_select);
        }
        keyboard::Key::Named(Named::ArrowRight) if shift => {
            move_cursor(this, api_cursor::move_right_select);
        }
        keyboard::Key::Named(Named::ArrowUp) if shift => {
            move_cursor(this, api_cursor::move_up_select);
        }
        keyboard::Key::Named(Named::ArrowDown) if shift => {
            move_cursor(this, api_cursor::move_down_select);
        }
        keyboard::Key::Named(Named::Home) if shift => {
            move_cursor(this, api_cursor::move_home_select);
        }
        keyboard::Key::Named(Named::End) if shift => {
            move_cursor(this, api_cursor::move_end_select);
        }

        // Обычное движение (без Shift) — сбрасывает выделение
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
            move_cursor(this, |doc| api_cursor::page_up(doc, n));
        }
        keyboard::Key::Named(Named::PageDown) => {
            let line_h = this.inner.base_size * 1.4;
            let n = (bounds.height / line_h) as usize;
            move_cursor(this, |doc| api_cursor::page_down(doc, n));
        }

        keyboard::Key::Named(Named::Backspace) => {
            edit(this, api_text::delete_before);
        }
        keyboard::Key::Named(Named::Delete) => {
            edit(this, api_text::delete_after);
        }
        keyboard::Key::Named(Named::Enter) => {
            edit(this, api_text::insert_newline);
        }
        _ => {
            if let Some(text) = text {
                if !cmd && !modifiers.alt() {
                    let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !filtered.is_empty() {
                        let text = filtered.clone();
                        edit(this, |doc| api_text::insert_text(doc, &text));
                    }
                }
            }
        }
    }

    auto_scroll(this, bounds);
    shell.request_redraw();
}
