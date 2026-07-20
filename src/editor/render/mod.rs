mod layout;
mod painter;

use crate::editor::cache::DocumentCache;
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;
use eframe::egui::{Color32, FontFamily, FontId, TextFormat, Ui};
use eframe::egui::text::{Galley, LayoutJob};
use std::sync::Arc;

pub use painter::paint;

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
            layout::source_layout(
                line,
                line_start,
                cache.lines.get(i),
                base_size,
                heading_size,
                &font_family,
                show_markers,
                ui.available_width(),
            )
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
