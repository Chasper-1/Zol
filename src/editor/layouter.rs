use crate::editor::markup::{LineMarkup, parse_line};
use eframe::egui::{Color32, FontFamily, FontId, TextFormat, text::LayoutJob};

fn append_compensated(
    job: &mut LayoutJob,
    markup_left: &str,
    content: &str,
    markup_right: Option<&str>,
    format: TextFormat,
) {
    // Вставляем невидимые символы, чтобы сдвинуть текст, сохранив индексы курсора
    if !markup_left.is_empty() {
        job.append(
            &"\u{200B}".repeat(markup_left.chars().count()),
            0.0,
            format.clone(),
        );
    }

    job.append(content, 0.0, format.clone());

    if let Some(right) = markup_right {
        job.append(&"\u{200B}".repeat(right.chars().count()), 0.0, format);
    }
}

pub fn render_line(
    job: &mut LayoutJob,
    line: &str,
    is_active: bool,
    base_size: f32,
    heading_size: f32,
    font_family: FontFamily,
    show_markup: bool,
) {
    if show_markup || is_active {
        // Режим отображения разметки: рисуем всё "как есть"[cite: 2]
        let is_heading = line.starts_with("# ");
        let format = TextFormat::simple(
            FontId::new(
                if is_heading { heading_size } else { base_size },
                font_family,
            ),
            Color32::WHITE,
        );
        job.append(line, 0.0, format);
    } else {
        // Режим скрытия: делегируем парсинг и компенсацию[cite: 2]
        match parse_line(line) {
            LineMarkup::Heading { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(heading_size, FontFamily::Proportional),
                    Color32::WHITE,
                );
                append_compensated(job, &marker, &content, None, format);
            }
            LineMarkup::Bold { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(base_size, font_family),
                    Color32::from_rgb(255, 100, 100),
                );
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Plain(text) => {
                let format = TextFormat::simple(
                    FontId::new(base_size, font_family),
                    Color32::from_rgb(180, 180, 180),
                );
                job.append(&text, 0.0, format);
            }
        }
    }
}
