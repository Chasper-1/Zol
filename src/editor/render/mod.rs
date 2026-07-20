pub mod font;
pub mod painter;
pub mod shape;

use crate::editor::cache::DocumentCache;
use crate::editor::layout;
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;

pub use shape::ShapedDocument;
pub use painter::{click_position, paint};

/// Собрать документ: вычислить TextRun'ы → сшейпить → готово к отрисовке.
///
/// Вызывает `font::init()` при первом вызове.
pub fn build(
    doc: &mut ShapedDocument,
    content: &str,
    cache: &DocumentCache,
    mode: EditMode,
    active_line: usize,
    theme: &EditorTheme,
    base_size: f32,
    heading_size: f32,
) {
    font::init(); // гарантируем инициализацию FontSystem

    let font_family = theme.text.font_family.as_deref().unwrap_or("sans-serif");

    let lines: Vec<&str> = content.split('\n').collect();
    let mut all_runs: Vec<Vec<layout::TextRun>> = Vec::with_capacity(lines.len());

    let mut line_start = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let show_markers = match mode {
            EditMode::Source => true,
            EditMode::Preview => false,
            EditMode::LivePreview => i == active_line,
        };

        let runs = if line.is_empty() {
            vec![layout::TextRun::new("\u{200B}", 0, crate::editor::theme::color::Rgba::new(0.5, 0.5, 0.5), base_size)]
        } else {
            layout::compute::compute_line_runs(
                line,
                line_start,
                cache.lines.get(i),
                base_size,
                heading_size,
                show_markers,
            )
        };

        all_runs.push(runs);
        line_start += line.len() + 1;
        if line_start > content.len() {
            line_start = content.len();
        }
    }

    font::with_font_system(|fs| {
        let shaped = shape::shape_document(&all_runs, fs, base_size, font_family);
        *doc = shaped;
    });
}
