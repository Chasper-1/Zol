use eframe::egui;
use rhai::Engine;
use std::fs;

use crate::editor::state::EditorState;
use crate::editor::theme;

pub struct FlintApp {
    state: EditorState,
}

impl FlintApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: EditorState) -> Self {
        // Применяем цвет фона из Rhai-темы к системному визуалу egui
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = state.theme.background.to_color32();
        cc.egui_ctx.set_visuals(visuals);

        Self { state }
    }
}

impl eframe::App for FlintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Верхняя панель (Тулбар)
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    let _ = fs::write("notes.md", &self.state.content);
                }
            });
        });

        // Центральная область редактора
        egui::CentralPanel::default().show(ctx, |ui| {
            let margin = self.state.theme.padding;
            egui::Frame::none().inner_margin(margin).show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.content)
                            .font(egui::FontId::new(
                                self.state.theme.text.size,
                                egui::FontFamily::Monospace,
                            ))
                            .desired_width(f32::INFINITY)
                            .lock_focus(true)
                            .code_editor(),
                    );

                    // Хоткей Ctrl + S
                    if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
                        let _ = fs::write("notes.md", &self.state.content);
                    }
                });
            });
        });
    }
}

// Точка входа для запуска UI, которую вызовет main
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
            // Загружаем тему из Rhai
            let src = fs::read_to_string("theme.rhai").unwrap_or_else(|_| {
                r#"
                let m = #{};
                m.editor = #{ padding: 12.0 };
                m.text = #{ size: 16.0, font_family: "Monospace" };
                m
                "#
                .to_string()
            });
            let engine = Engine::new();
            let ast = engine.compile(&src).expect("Rhai compile error");
            let rhai_map: rhai::Map = engine.eval_ast(&ast).expect("Rhai runtime error");
            let theme = theme::parse_theme(rhai_map);

            // Загружаем файл заметок
            let text = fs::read_to_string("notes.md").unwrap_or_default();
            let state = EditorState::new(theme, text);

            Ok(Box::new(FlintApp::new(cc, state)))
        }),
    )
}
