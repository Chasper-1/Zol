use crate::editor::{EditorState, EditorWidget, theme};
use gtk4::Application;
use gtk4::prelude::*;

use rhai::Engine;
use std::fs;

pub fn build_ui(app: &Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Flint Notes")
        .default_width(1024)
        .default_height(768)
        .build();

    let main_paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    main_paned.set_position(250);

    let sidebar_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let preview_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Читаем конфиг
    let src = fs::read_to_string("theme.rhai").expect("theme.rhai not found");
    let engine = Engine::new();
    let ast = engine.compile(&src).expect("Rhai compile error");
    let rhai_map: rhai::Map = engine.eval_ast(&ast).expect("Rhai runtime error");

    // Передаем карту в изолированный парсер темы
    let parsed_theme = theme::parse_theme(rhai_map);

    // Создаем независимое состояние редактора
    let editor_state = EditorState::new(parsed_theme);

    // Передаем стейт в виджет отрисовки
    let editor_area = EditorWidget::new(editor_state);

    let scrolled_window = gtk4::ScrolledWindow::builder()
        .child(&editor_area)
        .vexpand(true)
        .hexpand(true)
        .build();

    preview_container.append(&scrolled_window);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    window.set_child(Some(&main_paned));
    window.present();
}
