pub mod shape;

use crate::editor::cache::DocumentCache;
use crate::editor::layout;
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;

pub use shape::ShapedDocument;

/// Собрать документ: вычислить TextRun'ы → сшейпить → готово к отрисовке.
///
/// `scroll_y` — вертикальный сдвиг прокрутки (пиксели). cosmic-text
/// сформирует только видимое окно `[scroll_y, scroll_y + viewport_height]`.
///
/// `viewport_height` — если Some, ограничивает шейпинг видимой областью.
///
/// Вызывает `crate::editor::font::init()` при первом вызове.
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
) {
    crate::editor::font::init();

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
                theme,
            )
        };

        all_runs.push(runs);
        line_start += line.len() + 1;
        if line_start > content.len() {
            line_start = content.len();
        }
    }

        crate::editor::font::with_font_system(|fs| {
        *doc = shape::shape_document(
            &all_runs,
            fs,
            base_size,
            font_family,
            scroll_y,
            viewport_height,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::cache::DocumentCache;
    use crate::editor::font;
    use crate::editor::state::EditMode;
    use crate::editor::theme::EditorTheme;

    /// Regression: прямой вызов `build()` не должен deadlock'ать.
    #[test]
    fn build_does_not_deadlock() {
        font::init();
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
        let cache = DocumentCache::default();
        let theme = EditorTheme::default();
        // прямой вызов — без внешнего with_font_system
        build(
            &mut doc,
            "hello",
            &cache,
            EditMode::LivePreview,
            0,
            &theme,
            14.0,
            24.0,
            0.0,
            None,
        );
        assert!(
            doc.line_count() > 0,
            "doc should be shaped after build"
        );
    }

    #[test]
    fn build_multiline() {
        font::init();
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
        let cache = DocumentCache::default();
        let theme = EditorTheme::default();
        build(
            &mut doc,
            "line 1\nline 2\nline 3",
            &cache,
            EditMode::Source,
            0,
            &theme,
            14.0,
            24.0,
            0.0,
            None,
        );
        assert_eq!(doc.line_count(), 3);
    }

    #[test]
    fn build_empty_content() {
        font::init();
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
        let cache = DocumentCache::default();
        let theme = EditorTheme::default();
        build(
            &mut doc,
            "",
            &cache,
            EditMode::LivePreview,
            0,
            &theme,
            14.0,
            24.0,
            0.0,
            None,
        );
        assert_eq!(doc.line_count(), 1); // one empty line
    }

    #[test]
    fn build_with_scroll() {
        font::init();
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
        let cache = DocumentCache::default();
        let theme = EditorTheme::default();
        build(
            &mut doc,
            "hello\nworld",
            &cache,
            EditMode::Source,
            0,
            &theme,
            14.0,
            24.0,
            100.0,
            Some(200.0),
        );
        // with scroll, cosmic-text only shapes what's visible
        // but total_height should still work
        assert!(doc.total_height() >= 0.0);
    }
}
