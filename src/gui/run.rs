use eframe::egui;
use rhai::Engine;
use std::fs;
use crate::editor::state::EditorState;
use crate::editor::theme;
use crate::gui::app::FlintApp;

pub fn run_app() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Flint Notes")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Flint Notes",
        options,
        Box::new(|cc| {
            let src = fs::read_to_string("theme.rhai").unwrap_or_else(|_| {
                r#"#{ editor: #{ padding: 10.0, radius: 16.0, background: rgba(39, 46, 51, 0.9), }, text: #{ size: 14.0, color: rgba(205, 214, 244, 1.0), font_family: "SansSerif", } }"#.to_string()
            });

            let mut engine = Engine::new();
            
            // Регистрируем для всех комбинаций, которые может выдать парсер Rhai
            engine.register_fn("rgba", |r: i64, g: i64, b: i64, a: f64| {
                format!("rgba({}, {}, {}, {})", r, g, b, a)
            });
            
            let ast = engine.compile(&src).expect("Rhai compile error");
            let rhai_map: rhai::Map = engine.eval_ast(&ast).expect("Rhai runtime error");
            let theme = theme::parse_theme(rhai_map);

            let text = fs::read_to_string("notes.md").unwrap_or_default();
            let state = EditorState::new(theme, text);

            Ok(Box::new(FlintApp::new(cc, state)))
        }),
    )
}