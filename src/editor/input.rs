use crate::api;
use crate::editor::editor_widget::EditorWidget;
use crate::editor::state::EditMode;
use eframe::egui;
use std::fs;

pub fn handle_input(
    widget: &mut EditorWidget,
    mode: EditMode,
    ui: &egui::Ui,
) -> bool {
    if mode == EditMode::Preview {
        return false;
    }

    let input = ui.input(|i| {
        let pressed = |k: egui::Key| i.key_pressed(k);

        let mut text_events: Vec<String> = Vec::new();
        for event in &i.events {
            if let egui::Event::Text(t) = event {
                if !i.modifiers.command && !i.modifiers.alt {
                    text_events.push(t.clone());
                }
            }
        }

        let command = i.modifiers.command;

        (command, text_events, pressed(egui::Key::S), pressed(egui::Key::Backspace),
         pressed(egui::Key::Delete), pressed(egui::Key::Enter),
         pressed(egui::Key::ArrowLeft), pressed(egui::Key::ArrowRight),
         pressed(egui::Key::ArrowUp), pressed(egui::Key::ArrowDown),
         pressed(egui::Key::Home), pressed(egui::Key::End))
    });

    let (command, text_events, key_s, key_backspace, key_delete, key_enter,
         key_left, key_right, key_up, key_down, key_home, key_end) = input;

    let mut dirty = false;

    if command && key_s {
        let _ = fs::write("notes.md", widget.content());
        return false;
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

    if key_left {
        api::cursor::move_left(widget);
    }

    if key_right {
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
