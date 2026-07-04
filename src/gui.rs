use eframe::egui;
use rhai::Engine;
use std::fs;

use crate::editor::state::{EditMode, EditorState};
use crate::editor::theme;

pub struct FlintApp {
    state: EditorState,
}

impl FlintApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: EditorState) -> Self {
        let mut visuals = egui::Visuals::dark();

        // Используем цвет фона из конфига
        visuals.panel_fill = state.theme.background.to_color32();

        // Прямое присвоение в публичное поле corner_radius структуры WidgetVisuals
        let radius_u8 = state.theme.radius.round() as u8;
        let target_radius = egui::CornerRadius::same(radius_u8);

        visuals.widgets.noninteractive.corner_radius = target_radius;
        visuals.widgets.inactive.corner_radius = target_radius;
        visuals.widgets.hovered.corner_radius = target_radius;
        visuals.widgets.active.corner_radius = target_radius;

        cc.egui_ctx.set_visuals(visuals);
        Self { state }
    }
}

impl eframe::App for FlintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ТУЛБАР С РЕЖИМАМИ (Кнопку "Save" вырезали к чертям)
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Переключалка режимов
                ui.selectable_value(&mut self.state.mode, EditMode::Preview, "👁 Preview");
                ui.selectable_value(&mut self.state.mode, EditMode::LivePreview, "⚡ Live");
                ui.selectable_value(&mut self.state.mode, EditMode::Source, "📝 Source");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let margin = self.state.theme.padding;
            egui::Frame::NONE.inner_margin(margin).show(ui, |ui| {
                // Адаптация под egui 0.31: используем id_salt вместо id_source и auto_shrink для растягивания на всё окно
                egui::ScrollArea::vertical()
                    .id_salt("editor_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // Передаем сохраненный с прошлого кадра индекс в леяутер
                        let mut smart_layouter = crate::editor::layouter::create_smart_layouter(
                            self.state.mode,
                            self.state.active_line_index,
                            self.state.theme.clone(),
                        );

                        let is_editable = self.state.mode != EditMode::Preview;
                        let text_edit = egui::TextEdit::multiline(&mut self.state.content)
                            .desired_width(f32::INFINITY)
                            .min_size(ui.available_size()) // Растягиваем на всё окно
                            .frame(false) // Убираем только уродливый дефолтный фон
                            .lock_focus(true)
                            .interactive(is_editable)
                            .text_color(self.state.theme.text.color.to_color32())
                            .layouter(&mut smart_layouter);

                        // Достаем output, в котором egui отдает всю инфу о прошедшем рендере
                        let output = text_edit.show(ui);
                        let response = output.response;

                        // Магия: если редактор в фокусе, вытаскиваем курсор через официальный state
                        if let Some(state) = egui::TextEdit::load_state(ctx, response.id) {
                            if let Some(range) = state.cursor.range(&output.galley) {
                                let char_index = range.primary.ccursor.index;

                                // БЕЗОПАСНЫЙ СЧЕТ: берем символы, а не байты, чтобы не ломать кириллицу
                                let current_line = self
                                    .state
                                    .content
                                    .chars()
                                    .take(char_index)
                                    .filter(|&c| c == '\n')
                                    .count();

                                // Если юзер перешел на другую строку, сохраняем её и триггерим перерисовку
                                if self.state.active_line_index != Some(current_line) {
                                    self.state.active_line_index = Some(current_line);
                                    ctx.request_repaint();
                                }
                            }
                        }

                        // Если фокус потерян, сбрасываем активную строку
                        if !response.has_focus() && self.state.active_line_index.is_some() {
                            self.state.active_line_index = None;
                            ctx.request_repaint();
                        }

                        ctx.input(|i| {
                            if i.modifiers.command {
                                if i.key_pressed(egui::Key::Num1) {
                                    self.state.mode = EditMode::Preview;
                                }
                                if i.key_pressed(egui::Key::Num2) {
                                    self.state.mode = EditMode::LivePreview;
                                }
                                if i.key_pressed(egui::Key::Num3) {
                                    self.state.mode = EditMode::Source;
                                }
                                if i.key_pressed(egui::Key::S) {
                                    let _ = fs::write("notes.md", &self.state.content);
                                }
                            }
                        });
                    });
            });
        });
    }
}

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
                r#"
                #{
                    editor: #{
                        padding: 10.0,
                        radius: 16.0,
                        background: rgba(39, 46, 51, 0.9),
                    },
                    text: #{
                        size: 14.0,
                        color: rgba(205, 214, 244, 1.0),
                        font_family: "SansSerif",
                    }
                }
                "#
                .to_string()
            });

            let mut engine = Engine::new();

            // МАГИЯ: Учим Rhai понимать человеческую функцию rgba()
            engine.register_fn("rgba", |r: f64, g: f64, b: f64, a: f64| {
                format!("rgba({}, {}, {}, {})", r, g, b, a)
            });
            // На случай, если целые числа передадут как int
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
