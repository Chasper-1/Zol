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

fn append_compensated(
    job: &mut LayoutJob,
    markup_left: &str,
    content: &str,
    markup_right: Option<&str>,
    format: egui::TextFormat,
) {
    // 1. Считаем и компенсируем левый маркер
    if !markup_left.is_empty() {
        let compensation = "\u{200B}".repeat(markup_left.chars().count());
        job.append(&compensation, 0.0, format.clone());
    }

    // 2. Добавляем основной контент (видимый текст)
    job.append(content, 0.0, format.clone());

    // 3. Считаем и компенсируем правый маркер (если он передан как argument)
    if let Some(right) = markup_right {
        let compensation = "\u{200B}".repeat(right.chars().count());
        job.append(&compensation, 0.0, format);
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
    if show_markup || is_active {
        // Режим показа: рисуем всё как есть
        let format = egui::TextFormat::simple(
            egui::FontId::new(
                if line.starts_with("# ") {
                    heading_size
                } else {
                    base_size
                },
                font_family,
            ),
            egui::Color32::WHITE,
        );
        job.append(line, 0.0, format);
    } else {
        // Режим скрытия: используем нашу отдельную функцию для компенсации
        if line.starts_with("# ") {
            let format = egui::TextFormat::simple(
                egui::FontId::new(heading_size, egui::FontFamily::Proportional),
                egui::Color32::WHITE,
            );
            // Передаем левый маркер "# ", контент, и None для правого маркера
            append_compensated(job, "# ", &line[2..], None, format);
        } else if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
            let format = egui::TextFormat::simple(
                egui::FontId::new(base_size, font_family),
                egui::Color32::from_rgb(255, 100, 100),
            );
            // Передаем левый "**", контент, и правый "**"
            append_compensated(job, "**", &line[2..line.len() - 2], Some("**"), format);
        } else {
            // Обычный текст
            let format = egui::TextFormat::simple(
                egui::FontId::new(base_size, font_family),
                egui::Color32::from_rgb(180, 180, 180),
            );
            job.append(line, 0.0, format);
        }
    }
}
