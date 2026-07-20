use crate::api;
use crate::editor::editor_widget::EditorWidget;
use crate::editor::state::EditMode;
use eframe::egui;
use std::fs;

pub fn handle_input(widget: &mut EditorWidget, mode: EditMode, ui: &egui::Ui) -> bool {
    if mode == EditMode::Preview {
        return false;
    }

    let input = ui.input(|i| {
        let pressed = |k: egui::Key| i.key_pressed(k);

        let mut text_events: Vec<String> = Vec::new();
        let mut paste: Option<String> = None;

        for event in &i.events {
            match event {
                egui::Event::Text(t) => {
                    if !i.modifiers.command && !i.modifiers.alt {
                        text_events.push(t.clone());
                    }
                }
                egui::Event::Paste(t) => {
                    paste = Some(t.clone());
                }
                _ => {}
            }
        }

        let command = i.modifiers.command;

        (
            command,
            text_events,
            paste,
            pressed(egui::Key::S),
            pressed(egui::Key::Backspace),
            pressed(egui::Key::Delete),
            pressed(egui::Key::Enter),
            pressed(egui::Key::ArrowLeft),
            pressed(egui::Key::ArrowRight),
            pressed(egui::Key::ArrowUp),
            pressed(egui::Key::ArrowDown),
            pressed(egui::Key::Home),
            pressed(egui::Key::End),
        )
    });

    let (
        command,
        text_events,
        paste,
        key_s,
        key_backspace,
        key_delete,
        key_enter,
        key_left,
        key_right,
        key_up,
        key_down,
        key_home,
        key_end,
    ) = input;

    let mut dirty = false;

    if command && key_s {
        match fs::write("notes.md", widget.content()) {
            Ok(_) => {
                eprintln!("[Flint] Saved notes.md");
            }
            Err(e) => {
                eprintln!("[Flint] Ошибка сохранения: {}", e);
                // Визуальный сигнал: показываем ошибку через контекст egui
                // Запрашиваем перерисовку
                ui.ctx().request_repaint();
            }
        }
        return false;
    }

    if let Some(t) = paste {
        api::text::insert_at_cursor(widget, &t);
        return true;
    }

    for t in &text_events {
        api::text::insert_at_cursor(widget, t);
        dirty = true;
    }

    if key_backspace && !command {
        api::text::delete_before_cursor(widget);
        dirty = true;
    }

    if key_delete && !command {
        api::text::delete_after_cursor(widget);
        dirty = true;
    }

    if key_enter && !command {
        api::text::newline(widget);
        dirty = true;
    }

    if command && key_left {
        api::cursor::move_word_left(widget);
    } else if key_left {
        api::cursor::move_left(widget);
    }

    if command && key_right {
        api::cursor::move_word_right(widget);
    } else if key_right {
        api::cursor::move_right(widget);
    }

    if key_up {
        api::cursor::move_up(widget);
    }

    if key_down {
        api::cursor::move_down(widget);
    }

    if key_home {
        api::cursor::move_home(widget);
    }

    if key_end {
        api::cursor::move_end(widget);
    }

    dirty
}
