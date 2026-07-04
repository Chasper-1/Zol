use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use std::sync::Arc;

pub fn create_smart_layouter(
    mode: EditMode,
    active_line_index: Option<usize>,
    theme: EditorTheme,
) -> impl Fn(&egui::Ui, &str, f32) -> Arc<egui::Galley> {
    let base_size = theme.text.size;
    let heading_size = base_size * 1.6;

    // Умный маппинг строки из конфига в гарантированно существующие семейства шрифтов egui
    let font_family = match theme.text.font_family.to_lowercase().as_str() {
        "monospace" | "mono" => egui::FontFamily::Monospace,
        "sansserif" | "sans-serif" | "sans" | "proportional" => egui::FontFamily::Proportional,
        // Если написано что-то другое, пытаемся взять как кастомное имя
        _ => egui::FontFamily::Name(Arc::from(theme.text.font_family.clone())),
    };

    move |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;

        let lines: Vec<&str> = text.split('\n').collect();

        for (idx, line) in lines.iter().enumerate() {
            let is_active = Some(idx) == active_line_index;

            match mode {
                EditMode::Source => {
                    let format = egui::TextFormat::simple(
                        egui::FontId::new(base_size, font_family.clone()),
                        egui::Color32::from_rgb(200, 200, 200),
                    );
                    job.append(line, 0.0, format);
                }
                EditMode::Preview => {
                    render_pretty_line(
                        &mut job,
                        line,
                        base_size,
                        heading_size,
                        font_family.clone(),
                    );
                }
                EditMode::LivePreview => {
                    if is_active {
                        let mut format = egui::TextFormat::simple(
                            egui::FontId::new(base_size, font_family.clone()),
                            egui::Color32::from_rgb(255, 215, 0),
                        );
                        format.background =
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10);
                        job.append(line, 0.0, format);
                    } else {
                        render_pretty_line(
                            &mut job,
                            line,
                            base_size,
                            heading_size,
                            font_family.clone(),
                        );
                    }
                }
            }

            if idx < lines.len() - 1 {
                job.append(
                    "\n",
                    0.0,
                    egui::TextFormat::simple(
                        egui::FontId::new(base_size, font_family.clone()),
                        egui::Color32::TRANSPARENT,
                    ),
                );
            }
        }

        ui.fonts(|f| f.layout_job(job))
    }
}

fn render_pretty_line(
    job: &mut LayoutJob,
    line: &str,
    base_size: f32,
    heading_size: f32,
    font_family: egui::FontFamily,
) {
    if line.starts_with("# ") {
        let format = egui::TextFormat::simple(
            egui::FontId::new(heading_size, egui::FontFamily::Proportional),
            egui::Color32::WHITE,
        );
        job.append(&line[2..], 0.0, format);
    } else if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
        let format = egui::TextFormat::simple(
            egui::FontId::new(base_size, font_family.clone()),
            egui::Color32::from_rgb(255, 100, 100),
        );
        job.append(&line[2..line.len() - 2], 0.0, format);
    } else {
        let format = egui::TextFormat::simple(
            egui::FontId::new(base_size, font_family),
            egui::Color32::from_rgb(180, 180, 180),
        );
        job.append(line, 0.0, format);
    }
}
