use crate::editor::editor_widget::EditorWidget;
use crate::editor::state::{EditMode, EditorState};
use crate::editor::theme::Rgba;
use eframe::egui;

fn rgba_to_color32(c: &Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

pub struct ZolApp {
    state: EditorState,
    editor: EditorWidget,
    last_mode: EditMode,
}

impl ZolApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: EditorState) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = rgba_to_color32(&state.theme.background);

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

impl eframe::App for ZolApp {
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
