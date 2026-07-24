use super::shaped_doc::ShapedDocument;
use super::shape::shape_document;
use crate::cache::DocumentCache;
use crate::layout;
use crate::state::EditMode;
use crate::theme::EditorTheme;
use crate::Viewport;

/// Собрать документ: вычислить TextRun'ы → сшейпить → готово к отрисовке.
///
/// Если передан `viewport`, для строк вне viewport используется
/// простой (неокрашенный) TextRun — без cache-лукапа и markup-обработки.
pub fn build(
    doc: &mut ShapedDocument,
    content: &str,
    cache: &DocumentCache,
    mode: EditMode,
    active_line: usize,
    theme: &EditorTheme,
    base_size: f32,
    heading_size: f32,
    scroll_y: f32,
    viewport_height: Option<f32>,
    viewport: Option<&Viewport>,
) {
    crate::font::init();

    let font_family = theme.text.font_family.as_deref().unwrap_or("sans-serif");
    let default_color = theme.text.color;

    let lines: Vec<&str> = content.split('\n').collect();
    let mut all_runs: Vec<Vec<layout::TextRun>> = Vec::with_capacity(lines.len());

    // Диапазон строк, которые нужно полноценно обрабатывать.
    let visible_range = viewport.map(|vp| vp.first_line..=vp.last_line);

    let mut line_start = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let is_visible = visible_range.as_ref().is_none_or(|r| r.contains(&i));

        let runs = if line.is_empty() {
            vec![layout::TextRun::new(
                "\u{200B}",
                0,
                crate::theme::color::Rgba::new(0.5, 0.5, 0.5),
                base_size,
            )]
        } else if is_visible {
            let show_markers = match mode {
                EditMode::Source => true,
                EditMode::Preview => false,
                EditMode::LivePreview => i == active_line,
            };
            layout::compute::compute_line_runs(
                line,
                line_start,
                cache.lines.get(i),
                base_size,
                heading_size,
                show_markers,
                theme,
            )
        } else {
            // Строка вне viewport — только базовый цвет, без семантики
            vec![layout::TextRun::new(line, 0, default_color, base_size)]
        };

        all_runs.push(runs);
        line_start += line.len() + 1;
        if line_start > content.len() {
            line_start = content.len();
        }
    }

    crate::font::with_font_system(|fs| {
        *doc = shape_document(&all_runs, fs, base_size, font_family, scroll_y, viewport_height);
    });
}
