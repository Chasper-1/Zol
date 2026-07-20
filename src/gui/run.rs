use crate::editor::state::EditorState;
use crate::editor::theme;
use crate::gui::app::ZolApp;
use eframe::egui;
use rhai::Engine;
use std::fs;

pub fn run_app() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Zol")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Zol",
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

            let theme = match engine.compile(&src) {
                Ok(ast) => match engine.eval_ast::<rhai::Map>(&ast) {
                    Ok(rhai_map) => theme::parse_theme(rhai_map),
                    Err(e) => {
                        eprintln!(
                            "[Zol] Ошибка выполнения темы Rhai: {}. Использую тему по умолчанию",
                            e
                        );
                        theme::EditorTheme::default()
                    }
                },
                Err(e) => {
                    eprintln!(
                        "[Zol] Ошибка компиляции темы Rhai: {}. Использую тему по умолчанию",
                        e
                    );
                    theme::EditorTheme::default()
                }
            };

            let text = match fs::read_to_string("notes.zml") {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[Zol] notes.zml не найден ({}), создаю пустой документ", e);
                    String::new()
                }
            };
            let state = EditorState::new(theme, text);

            Ok(Box::new(ZolApp::new(cc, state)))
        }),
    )
}
