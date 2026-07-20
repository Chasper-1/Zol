use crate::editor::cache::MarkupCache;
use crate::editor::markup::segment::{
    STYLE_BOLD, STYLE_CODE, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_SUBSCRIPT, STYLE_SUPERSCRIPT,
};
use eframe::egui::text::LayoutJob;
use eframe::egui::{Align, Color32, FontFamily, FontId, Stroke, TextFormat};

fn append_compensated(
    job: &mut LayoutJob,
    left: usize,
    content: &str,
    right: usize,
    format: TextFormat,
) {
    if left > 0 {
        job.append(&"\u{200B}".repeat(left), 0.0, format.clone());
    }

    job.append(content, 0.0, format.clone());

    if right > 0 {
        job.append(&"\u{200B}".repeat(right), 0.0, format);
    }
}

pub fn render_line(
    job: &mut LayoutJob,
    line: &str,
    cache: &MarkupCache,
    base_size: f32,
    heading_size: f32,
    font_family: FontFamily,
    show_markup: bool,
) {
    let default_format = TextFormat::simple(
        FontId::new(base_size, font_family.clone()),
        Color32::from_rgb(180, 180, 180),
    );

    if show_markup {
        job.append(line, 0.0, default_format);
        return;
    }

    if let Some(content) = line.strip_prefix("# ") {
        let format = TextFormat::simple(FontId::new(heading_size, font_family), Color32::WHITE);
        job.append(content, 0.0, format);
        return;
    }

    if cache.segments.is_empty() {
        job.append(line, 0.0, default_format);
        return;
    }

    for seg in &cache.segments {
        let style = seg.style;
        let mut format = default_format.clone();

        if style & STYLE_BOLD != 0 {
            format.color = Color32::from_rgb(255, 100, 100);
        }

        if style & STYLE_ITALIC != 0 {
            format.color = Color32::from_rgb(100, 200, 255);
            format.italics = true;
        }

        if style & STYLE_STRIKETHROUGH != 0 {
            format.color = Color32::from_rgb(200, 150, 150);
            format.strikethrough = Stroke::new(1.0, Color32::from_rgb(200, 150, 150));
        }

        if style & STYLE_SUPERSCRIPT != 0 {
            format.font_id = FontId::new(base_size * 0.7, format.font_id.family);
            format.color = Color32::from_rgb(150, 255, 150);
            format.valign = Align::TOP;
        }

        if style & STYLE_SUBSCRIPT != 0 {
            format.font_id = FontId::new(base_size * 0.7, format.font_id.family);
            format.color = Color32::from_rgb(255, 200, 100);
            format.valign = Align::BOTTOM;
        }

        if style & STYLE_CODE != 0 {
            format.font_id = FontId::new(base_size, FontFamily::Monospace);
            format.color = Color32::from_rgb(200, 200, 200);
        }

        append_compensated(
            job,
            seg.left_marker_len,
            &seg.text,
            seg.right_marker_len,
            format,
        );
    }
}
