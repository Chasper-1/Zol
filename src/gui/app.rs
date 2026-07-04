use eframe::egui;
use std::fs;
use crate::editor::state::{EditMode, EditorState};
use crate::editor::layouter;

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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.state.mode, EditMode::Preview, "👁 Preview");
                ui.selectable_value(&mut self.state.mode, EditMode::LivePreview, "⚡ Live");
                ui.selectable_value(&mut self.state.mode, EditMode::Source, "📝 Source");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::NONE.inner_margin(self.state.theme.padding).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("editor_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let theme = self.state.theme.clone();
                        let base_size = theme.text.size;
                        let heading_size = base_size * 1.6;
                        let font_family = theme.text.font_family.map_or(
                            egui::FontFamily::Proportional, 
                            |name| egui::FontFamily::Name(std::sync::Arc::from(name.as_str()))
                        );

                        let mut layouter_func = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let mut job = egui::text::LayoutJob::default();
                            job.wrap.max_width = wrap_width;
                            
                            for (idx, line) in text.split('\n').enumerate() {
                                let is_active = Some(idx) == self.state.active_line_index;
                                let show_markup = self.state.mode == EditMode::Source 
                                    || (self.state.mode == EditMode::LivePreview && is_active);

                                layouter::render_line(
                                    &mut job, line, is_active, base_size, heading_size, 
                                    font_family.clone(), show_markup
                                );
                                job.append("\n", 0.0, egui::TextFormat::default());
                            }
                            ui.fonts(|f| f.layout_job(job))
                        };

                        let text_edit = egui::TextEdit::multiline(&mut self.state.content)
                            .desired_width(f32::INFINITY)
                            .min_size(ui.available_size())
                            .frame(false)
                            .lock_focus(true)
                            .interactive(self.state.mode != EditMode::Preview)
                            .text_color(self.state.theme.text.color.to_color32())
                            .layouter(&mut layouter_func);

                        let output = text_edit.show(ui);
                        
                        if let Some(state) = egui::TextEdit::load_state(ctx, output.response.id) {
                            if let Some(range) = state.cursor.range(&output.galley) {
                                let line = self.state.content.chars().take(range.primary.ccursor.index).filter(|&c| c == '\n').count();
                                if self.state.active_line_index != Some(line) {
                                    self.state.active_line_index = Some(line);
                                    ctx.request_repaint();
                                }
                            }
                        }

                        if output.response.has_focus() {
                            ctx.input(|i| {
                                let ctrl = i.modifiers.ctrl;
                                let right = i.key_pressed(egui::Key::ArrowRight);
                                let left = i.key_pressed(egui::Key::ArrowLeft);
                        
                                if ctrl && (right || left) {
                                    if let Some(line) = self.state.content.lines().nth(self.state.active_line_index.unwrap_or(0)) {
                                        // Вызываем нашу функцию из layouter
                                        crate::editor::layouter::adjust_cursor_for_markup(ctx, output.response.id, line, right, &output.galley);
                                    }
                                }
                            });
                        }

                        ctx.input(|i| {
                            if i.modifiers.command {
                                if i.key_pressed(egui::Key::Num1) { self.state.mode = EditMode::Preview; }
                                if i.key_pressed(egui::Key::Num2) { self.state.mode = EditMode::LivePreview; }
                                if i.key_pressed(egui::Key::Num3) { self.state.mode = EditMode::Source; }
                                if i.key_pressed(egui::Key::S) { let _ = fs::write("notes.md", &self.state.content); }
                            }
                        });
                    });
            });
        });
    }
}