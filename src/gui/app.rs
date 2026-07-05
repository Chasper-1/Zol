use crate::editor::layouter;
use crate::editor::markup::parse_document;
use crate::editor::state::{EditMode, EditorState};
use eframe::egui;
use std::fs;

pub struct FlintApp {
    state: EditorState,
}

impl FlintApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: EditorState) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = state.theme.background.to_color32();

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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Панель инструментов
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.state.mode, EditMode::Preview, "👁 Preview");
                ui.selectable_value(&mut self.state.mode, EditMode::LivePreview, "⚡ Live");
                ui.selectable_value(&mut self.state.mode, EditMode::Source, "📝 Source");
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::Frame::NONE
                .inner_margin(self.state.theme.padding)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("editor_scroll")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let theme = self.state.theme.clone();
                            let base_size = theme.text.size;
                            let heading_size = base_size * 1.6;

                            self.state.document_cache = parse_document(&self.state.content);

                            let font_family = theme
                                .text
                                .font_family
                                .map_or(egui::FontFamily::Proportional, |name| {
                                    egui::FontFamily::Name(std::sync::Arc::from(name.as_str()))
                                });

                            let mut layouter_func =
                                |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                    let text_str = text.as_str();
                                    let mut job = egui::text::LayoutJob::default();
                                    job.wrap.max_width = wrap_width;

                                    let lines: Vec<&str> = text_str.split('\n').collect();
                                    let line_count = lines.len();

                                    for (idx, line) in lines.iter().enumerate() {
                                        let show_markup = self.state.mode == EditMode::Source;
                                        let cache = &self.state.document_cache.lines[idx];

                                        layouter::render_line(
                                            &mut job,
                                            line,
                                            cache,
                                            base_size,
                                            heading_size,
                                            font_family.clone(),
                                            show_markup,
                                        );

                                        if idx < line_count - 1 {
                                            job.append("\n", 0.0, egui::TextFormat::default());
                                        }
                                    }
                                    ui.fonts_mut(|f| f.layout_job(job))
                                };

                            let text_edit = egui::TextEdit::multiline(&mut self.state.content)
                                .desired_width(f32::INFINITY)
                                .min_size(ui.available_size())
                                .frame(egui::Frame::NONE)
                                .lock_focus(true)
                                .interactive(self.state.mode != EditMode::Preview)
                                .text_color(self.state.theme.text.color.to_color32())
                                .layouter(&mut layouter_func);

                            let output = text_edit.show(ui);

                            // Горячие клавиши
                            ui.ctx().input(|i| {
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
