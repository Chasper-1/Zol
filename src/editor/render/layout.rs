use crate::editor::cache::MarkupCache;
use crate::editor::utils::line_utils;
use crate::editor::markup::segment::{
    STYLE_BOLD, STYLE_CODE, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA,
    STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_SUBSCRIPT,
    STYLE_SUPERSCRIPT, STYLE_UNDERLINE,
};
use eframe::egui::text::LayoutJob;
use eframe::egui::{Align, Color32, FontFamily, FontId, Stroke, TextFormat};

pub(super) fn source_layout(
    line: &str,
    line_start: usize,
    line_cache: Option<&MarkupCache>,
    base_size: f32,
    heading_size: f32,
    font_family: &FontFamily,
    show_markers: bool,
    available_width: f32,
) -> LayoutJob {
    let mut job = LayoutJob::default();

    if let Some(stripped) = line.strip_prefix("# ") {
        if show_markers {
            let hash_fmt = TextFormat::simple(
                FontId::new(heading_size, font_family.clone()),
                Color32::from_rgb(120, 120, 120),
            );
            job.append("# ", 0.0, hash_fmt);
        }
        let content_fmt = TextFormat::simple(
            FontId::new(heading_size, font_family.clone()),
            Color32::WHITE,
        );
        job.append(stripped, 0.0, content_fmt);
        return job;
    }

    let Some(cache) = line_cache else {
        let fmt = TextFormat::simple(
            FontId::new(base_size, font_family.clone()),
            Color32::from_rgb(200, 200, 200),
        );
        job.append(line, 0.0, fmt);
        return job;
    };

    if cache.segments.is_empty() {
        let fmt = TextFormat::simple(
            FontId::new(base_size, font_family.clone()),
            Color32::from_rgb(200, 200, 200),
        );
        job.append(line, 0.0, fmt);
        return job;
    }

    let mut last_end = 0usize;

    for seg in &cache.segments {
        let seg_start = seg.raw_start.saturating_sub(line_start);
        let seg_end = seg.raw_end.saturating_sub(line_start);

        if show_markers && seg_start > last_end && seg_start <= line.len() {
            let marker = &line[last_end..seg_start];
            if !marker.is_empty() {
                let fmt = TextFormat::simple(
                    FontId::new(base_size, font_family.clone()),
                    Color32::from_rgb(100, 100, 100),
                );
                job.append(marker, 0.0, fmt);
            }
        }

        if seg_start < line.len() {
            let end = seg_end.min(line.len());
            let segment_text = &line[seg_start..end];
            let fmt = segment_format(seg.style, base_size, heading_size, font_family);
            job.append(segment_text, 0.0, fmt);
        }

        last_end = seg_end.min(line.len());
    }

    if show_markers && last_end < line.len() {
        let marker = &line[last_end..];
        if !marker.is_empty() {
            let fmt = TextFormat::simple(
                FontId::new(base_size, font_family.clone()),
                Color32::from_rgb(100, 100, 100),
            );
            job.append(marker, 0.0, fmt);
        }
    }

    if let Some(cache) = line_cache
        && cache
            .segments
            .iter()
            .any(|s| s.style & STYLE_DISPLAY_FORMULA != 0)
    {
        job.wrap.max_width = available_width;
        job.halign = Align::Center;
    }

    job
}

fn segment_format(
    style: u32,
    base_size: f32,
    _heading_size: f32,
    font_family: &FontFamily,
) -> TextFormat {
    let mut format = TextFormat::simple(
        FontId::new(base_size, font_family.clone()),
        Color32::from_rgb(220, 220, 220),
    );

    if style & STYLE_BOLD != 0 {
        format.color = Color32::from_rgb(255, 100, 100);
    }
    if style & STYLE_ITALIC != 0 {
        format.color = Color32::from_rgb(100, 200, 255);
        format.italics = true;
    }
    if style & STYLE_STRIKETHROUGH != 0 {
        format.strikethrough = Stroke::new(1.0, format.color);
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
    if style & STYLE_UNDERLINE != 0 {
        format.underline = Stroke::new(1.0, format.color);
    }
    if style & STYLE_HIGHLIGHT != 0 {
        format.background = Color32::from_rgba_unmultiplied(255, 255, 0, 40);
    }
    if style & STYLE_INSERTION != 0 {
        format.color = Color32::from_rgb(100, 255, 100);
    }
    if style & STYLE_DELETION != 0 {
        format.color = Color32::from_rgb(255, 80, 80);
        format.strikethrough = Stroke::new(1.0, Color32::from_rgb(255, 80, 80));
    }
    if style & STYLE_COMMENT != 0 {
        format.color = Color32::from_rgb(140, 140, 140);
        format.italics = true;
    }
    if style & STYLE_FORMULA != 0 {
        format.font_id = FontId::new(base_size, FontFamily::Monospace);
        format.color = Color32::from_rgb(80, 220, 120);
    }
    if style & STYLE_DISPLAY_FORMULA != 0 {
        format.font_id = FontId::new(base_size * 1.3, FontFamily::Monospace);
        format.color = Color32::from_rgb(80, 220, 120);
    }
    format
}

pub(super) fn cursor_line_bounds(content: &str, line: usize) -> (usize, usize) {
    line_utils::line_bounds(content, line)
        .map(|b| (b.start, b.end))
        .unwrap_or((0, 0))
}
