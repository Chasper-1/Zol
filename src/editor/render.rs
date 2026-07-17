use crate::editor::cache::DocumentCache;
use crate::editor::cursor::Cursor;
use crate::editor::markup::segment::{
    STYLE_BOLD, STYLE_CODE, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA,
    STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_STRIKETHROUGH,
    STYLE_SUBSCRIPT, STYLE_SUPERSCRIPT, STYLE_UNDERLINE,
};
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;
use eframe::egui::text::{CCursor, Galley, LayoutJob};
use eframe::egui::{Align, Color32, FontFamily, FontId, Painter, Pos2, Stroke, TextFormat, Ui};
use std::sync::Arc;

pub struct Galleys {
    pub galleys: Vec<Option<Arc<Galley>>>,
    pub total_height: f32,
}

impl Galleys {
    pub fn new() -> Self {
        Self {
            galleys: Vec::new(),
            total_height: 0.0,
        }
    }
}

pub fn build(
    galleys: &mut Galleys,
    content: &str,
    cache: &DocumentCache,
    mode: EditMode,
    active_line: usize,
    ui: &Ui,
    theme: &EditorTheme,
    base_size: f32,
    heading_size: f32,
) {
    let font_family = theme
        .text
        .font_family
        .as_deref()
        .map(|name| FontFamily::Name(Arc::from(name)))
        .unwrap_or(FontFamily::Proportional);

    let lines: Vec<&str> = content.split('\n').collect();
    let num_lines = lines.len();
    let mut new_galleys = Vec::with_capacity(num_lines);
    let mut total_height = 0.0;

    let mut line_start = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let show_markers = match mode {
            EditMode::Source => true,
            EditMode::Preview => false,
            EditMode::LivePreview => i == active_line,
        };

        let job = if line.is_empty() {
            let mut job = LayoutJob::default();
            let fmt = TextFormat::simple(
                FontId::new(base_size, font_family.clone()),
                Color32::from_rgb(200, 200, 200),
            );
            job.append("\u{200B}", 0.0, fmt);
            job
        } else {
            source_layout(line, line_start, cache.lines.get(i), base_size, heading_size, &font_family, show_markers)
        };

        let galley = ui.fonts_mut(|f| f.layout_job(job));
        total_height += galley.size().y;
        new_galleys.push(Some(galley));

        line_start += line.len() + 1;
        if line_start > content.len() {
            line_start = content.len();
        }
    }

    galleys.galleys = new_galleys;
    galleys.total_height = total_height;
}

fn source_layout(
    line: &str,
    line_start: usize,
    line_cache: Option<&crate::editor::cache::MarkupCache>,
    base_size: f32,
    heading_size: f32,
    font_family: &FontFamily,
    show_markers: bool,
) -> LayoutJob {
    let mut job = LayoutJob::default();

    if line.starts_with("# ") {
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
        job.append(&line[2..], 0.0, content_fmt);
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

    if let Some(cache) = line_cache {
        if cache.segments.iter().any(|s| s.style & STYLE_DISPLAY_FORMULA != 0) {
            job.halign = Align::Center;
        }
    }

    job
}

fn segment_format(style: u32, base_size: f32, _heading_size: f32, font_family: &FontFamily) -> TextFormat {
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
        format.font_id = FontId::new(base_size, FontFamily::Monospace);
        format.color = Color32::from_rgb(80, 220, 120);
    }
    format
}

pub fn paint(
    galleys: &Galleys,
    cursor: &Cursor,
    painter: &Painter,
    origin: Pos2,
    text_color: Color32,
    content: &str,
    mode: EditMode,
) {
    let mut y_offset = origin.y;

    for (i, galley_opt) in galleys.galleys.iter().enumerate() {
        if let Some(galley) = galley_opt {
            let galley_size = galley.size();
            let pos = Pos2::new(origin.x, y_offset);

            painter.galley(pos, galley.clone(), text_color);

            if mode != EditMode::Preview && i == cursor.line {
                if let Some(cursor_rect) = cursor_rect(content, cursor, galley) {
                    let cursor_x = origin.x + cursor_rect.min.x;
                    let cursor_y = y_offset + cursor_rect.min.y;
                    let line_h = cursor_rect.height().max(galley_size.y * 0.8);

                    painter.line_segment(
                        [
                            Pos2::new(cursor_x, cursor_y),
                            Pos2::new(cursor_x, cursor_y + line_h),
                        ],
                        Stroke::new(2.0, text_color),
                    );
                }
            }

            y_offset += galley_size.y;
        }
    }
}

fn cursor_rect(content: &str, cursor: &Cursor, galley: &Galley) -> Option<eframe::egui::Rect> {
    let (line_start, line_end) = cursor_line_bounds(content, cursor.line);
    let byte_in_line = cursor.raw.saturating_sub(line_start);
    let line_text = &content[line_start..line_end];
    let char_idx = line_text[..byte_in_line.min(line_text.len())].chars().count();
    let egui_cursor = CCursor::new(char_idx);
    Some(galley.pos_from_cursor(egui_cursor))
}

fn cursor_line_bounds(content: &str, line: usize) -> (usize, usize) {
    let mut current = 0usize;
    let mut start = 0usize;
    for (i, c) in content.char_indices() {
        if current == line && c == '\n' {
            return (start, i);
        }
        if c == '\n' {
            current += 1;
            start = i + 1;
        }
    }
    if current == line {
        (start, content.len())
    } else {
        (0, 0)
    }
}
