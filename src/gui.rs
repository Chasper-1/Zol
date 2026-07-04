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
                        let mode = self.state.mode;
                        let active_line = self.state.active_line_index;
                        let theme = self.state.theme.clone();
                        let base_size = theme.text.size;
                        let heading_size = base_size * 1.6;
                        let font_family = if let Some(ref family_name) = theme.text.font_family {
                            egui::FontFamily::Name(std::sync::Arc::from(family_name.as_str()))
                        } else {
                            egui::FontFamily::Proportional // Дефолт, если в конфиге пусто
                        };

                        // Замыкание, которое вызывает твою render_line из layouter.rs
                        let mut layouter_func = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let mut job = egui::text::LayoutJob::default();
                            job.wrap.max_width = wrap_width;

                            let lines: Vec<&str> = text.split('\n').collect();
                            for (idx, line) in lines.iter().enumerate() {
                                let is_active = Some(idx) == active_line;
                                let show_markup = mode == EditMode::Source
                                    || (mode == EditMode::LivePreview && is_active);

                                // Вызов функции из layouter.rs
                                crate::editor::layouter::render_line(
                                    &mut job,
                                    line,
                                    is_active,
                                    base_size,
                                    heading_size,
                                    font_family.clone(),
                                    show_markup,
                                );

                                if idx < lines.len() - 1 {
                                    job.append("\n", 0.0, egui::TextFormat::default());
                                }
                            }
                            ui.fonts(|f| f.layout_job(job))
                        };

                        let is_editable = self.state.mode != EditMode::Preview;
                        let text_edit = egui::TextEdit::multiline(&mut self.state.content)
                            .desired_width(f32::INFINITY)
                            .min_size(ui.available_size())
                            .frame(false)
                            .lock_focus(true)
                            .interactive(is_editable)
                            .text_color(self.state.theme.text.color.to_color32())
                            .layouter(&mut layouter_func);

                        // Достаем output, в котором egui отдает всю инфу о прошедшем рендере
                        let output = text_edit.show(ui);
                        let response = output.response;

                        // Магия: если редактор в фокусе, вытаскиваем курсор через официальный state
                        if let Some(state) = egui::TextEdit::load_state(ctx, response.id) {
                            if let Some(range) = state.cursor.range(&output.galley) {
                                // Получаем байтовый индекс курсора
                                let cursor_index = range.primary.ccursor.index;

                                // БЕЗОПАСНЫЙ СПОСОБ: считаем переносы через итератор символов,
                                // который сам корректно обрабатывает UTF-8.
                                let current_line = self
                                    .state
                                    .content
                                    .chars()
                                    .take(cursor_index) // Берем символы до байтового индекса
                                    .filter(|&c| c == '\n')
                                    .count();

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
