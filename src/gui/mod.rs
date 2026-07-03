use crate::editor::EditorWidget;
use gtk4::Application;
use gtk4::prelude::*;

use rhai::Engine;
use std::fs;

// Подтягиваем функции парсинга, которые у тебя уже есть в проекте
// Предполагаю, что они лежат в модуле theme (исходя из твоих исходников)
pub mod theme_parser {
    use rhai::Map;

    // ДОБАВИЛИ pub ТУТ, чтобы структура и её поля были доступны в widget.rs
    #[derive(Default, Debug, Clone)]
    pub struct EditorTheme {
        pub padding: f32,
        pub radius: f32,
        pub background: String,
    }

    pub fn apply(map: Map) -> EditorTheme {
        let mut theme = EditorTheme::default();
        if let Some(editor) = map.get("editor") {
            let m = editor.clone().cast::<Map>();
            theme.padding = m
                .get("padding")
                .map(|v| v.clone().cast::<f32>())
                .unwrap_or(12.0);
            theme.radius = m
                .get("radius")
                .map(|v| v.clone().cast::<f32>())
                .unwrap_or(12.0);
            theme.background = m
                .get("background")
                .map(|v| v.clone().cast::<String>())
                .unwrap_or("#21212a".to_string());
        }
        theme
    }
}

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

    // =========================
    // ЧИТАЕМ И ПАРСИМ RHAI СТИЛИ
    // =========================
    let src = fs::read_to_string("theme.rhai").expect("theme.rhai not found");
    let engine = Engine::new();
    let ast = engine.compile(&src).expect("Rhai compile error");
    let rhai_map: rhai::Map = engine.eval_ast(&ast).expect("Rhai runtime error");

    // Применяем карту из скрипта к нашей структуре темы
    let editor_theme = theme_parser::apply(rhai_map);

    // Внедряем наш кастомный виджет вместо TextView и передаем ему тему!
    let editor_area = EditorWidget::new(editor_theme);

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
