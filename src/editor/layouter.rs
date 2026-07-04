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
    let heading_size = base_size * 1.6; // Заголовок всегда больше

    let font_family = match theme.text.font_family.to_lowercase().as_str() {
        "monospace" | "mono" => egui::FontFamily::Monospace,
        "sansserif" | "sans-serif" | "sans" | "proportional" => egui::FontFamily::Proportional,
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
                    render_line(
                        &mut job,
                        line,
                        is_active,
                        base_size,
                        heading_size,
                        font_family.clone(),
                        true,
                    );
                }
                EditMode::Preview => {
                    render_line(
                        &mut job,
                        line,
                        is_active,
                        base_size,
                        heading_size,
                        font_family.clone(),
                        false,
                    );
                }
                EditMode::LivePreview => {
                    // Передаем флаг активности: на активной строке покажем разметку, но размер заголовка не уменьшим!
                    render_line(
                        &mut job,
                        line,
                        is_active,
                        base_size,
                        heading_size,
                        font_family.clone(),
                        is_active,
                    );
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

// Универсальная функция отрисовки строки
fn render_line(
    job: &mut LayoutJob,
    line: &str,
    is_active: bool,
    base_size: f32,
    heading_size: f32,
    font_family: egui::FontFamily,
    show_markup: bool, // Показывать ли маркеры типа # или **
) {
    if line.starts_with("# ") {
        let format_visible = egui::TextFormat::simple(
            egui::FontId::new(heading_size, egui::FontFamily::Proportional),
            egui::Color32::WHITE,
        );
        let format_hidden = egui::TextFormat::simple(
            egui::FontId::new(heading_size, egui::FontFamily::Proportional),
            egui::Color32::TRANSPARENT, // Ставим прозрачность вместо удаления
        );

        if show_markup {
            job.append(line, 0.0, format_visible);
        } else {
            // Рисуем оба куска, но первый - прозрачный
            job.append("# ", 0.0, format_hidden); // Позиция сохранена!
            job.append(&line[2..], 0.0, format_visible);
        }
    } else if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
        let mut format = egui::TextFormat::simple(
            egui::FontId::new(base_size, font_family.clone()),
            egui::Color32::from_rgb(255, 100, 100),
        );
        if is_active && show_markup {
            format.background = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10);
            job.append(line, 0.0, format);
        } else if show_markup {
            job.append(line, 0.0, format);
        } else {
            job.append(&line[2..line.len() - 2], 0.0, format);
        }
    } else {
        let mut format = egui::TextFormat::simple(
            egui::FontId::new(base_size, font_family),
            egui::Color32::from_rgb(180, 180, 180),
        );
        if is_active && show_markup {
            format.background = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10);
        }
        job.append(line, 0.0, format);
    }
}
