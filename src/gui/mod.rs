use crate::editor::theme;
use gtk4::Application;
use gtk4::prelude::*;
use rhai::Engine;
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};
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

    // Читаем конфиг (логика темы остается для других частей UI)
    let src = fs::read_to_string("theme.rhai").expect("theme.rhai not found");
    let engine = Engine::new();
    let ast = engine.compile(&src).expect("Rhai compile error");
    let rhai_map: rhai::Map = engine.eval_ast(&ast).expect("Rhai runtime error");
    let parsed_theme = theme::parse_theme(rhai_map);

    // Создаем SourceBuffer — это сердце твоего нового редактора
    let buffer = Buffer::builder().build();

    // Подключаем Markdown (если нужен синтаксис)
    let lm = LanguageManager::default();
    if let Some(lang) = lm.language("markdown") {
        buffer.set_language(Some(&lang));
    }

    // Создаем View (виджет редактора)
    let editor_view = View::builder()
        .buffer(&buffer)
        .editable(true)
        .show_line_numbers(true)
        .wrap_mode(gtk4::WrapMode::Word)
        .highlight_current_line(true)
        .build();

    // Применяем темную тему, чтобы было похоже на Obsidian
    let sm = StyleSchemeManager::default();
    if let Some(scheme) = sm.scheme("classic-dark") {
        buffer.set_style_scheme(Some(&scheme));
    }

    let scrolled_window = gtk4::ScrolledWindow::builder()
        .child(&editor_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    preview_container.append(&scrolled_window);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    window.set_child(Some(&main_paned));
    window.present();
}
