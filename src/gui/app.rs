use crate::editor::editor_widget::EditorWidget;
use crate::editor::state::{EditMode, EditorState};
use eframe::egui;

pub struct FlintApp {
    state: EditorState,
    editor: EditorWidget,
    last_mode: EditMode,
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
        Self {
            editor: EditorWidget::new(&state.content),
            last_mode: state.mode,
            state,
        }
    }
}

impl eframe::App for FlintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.state.mode, EditMode::Preview, "👁 Preview");
                ui.selectable_value(&mut self.state.mode, EditMode::LivePreview, "⚡ Live");
                ui.selectable_value(&mut self.state.mode, EditMode::Source, "📝 Source");
            });
        });

        if self.state.mode != self.last_mode {
            self.editor.dirty = true;
            self.last_mode = self.state.mode;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            egui::Frame::NONE
                .inner_margin(self.state.theme.padding)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("editor_scroll")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.editor.ui(ui, &mut self.state);
                        });
                });
        });
    }
}
