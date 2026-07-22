use super::shared;
use super::style::text_run_for_style;
use crate::cache::MarkupCache;
use crate::layout::types::TextRun;
use crate::theme::EditorTheme;

/// Разобрать строку на стилизованные фрагменты.
#[allow(clippy::too_many_arguments)]
pub fn compute_line_runs(
    line: &str,
    line_start: usize,
    line_cache: Option<&MarkupCache>,
    base_size: f32,
    heading_size: f32,
    show_markers: bool,
    theme: &EditorTheme,
) -> Vec<TextRun> {
    if let Some(stripped) = line.strip_prefix("# ") {
        let mut runs = Vec::new();
        let color = if show_markers { shared::MARKER_GRAY } else { theme.background };
        runs.push(TextRun::new("# ", 0, color, heading_size));
        runs.push(TextRun::new(stripped, 0, shared::TEXT_WHITE, heading_size));
        return runs;
    }

    let Some(cache) = line_cache else {
        return vec![TextRun::new(line, 0, theme.text.color, base_size)];
    };

    if cache.segments.is_empty() {
        return vec![TextRun::new(line, 0, theme.text.color, base_size)];
    }

    let mut runs = Vec::new();
    let mut last_end = 0usize;

    for seg in &cache.segments {
        let seg_start = seg.raw_start.saturating_sub(line_start);
        let seg_end = seg.raw_end.saturating_sub(line_start);

        if seg_start > last_end && seg_start <= line.len() {
            let marker = &line[last_end..seg_start];
            if !marker.is_empty() {
                let color = if show_markers { shared::MARKER_GRAY } else { theme.background };
                runs.push(TextRun::new(marker, 0, color, base_size));
            }
        }

        if seg_start < line.len() {
            let end = seg_end.min(line.len());
            let segment_text = &line[seg_start..end];
            runs.push(text_run_for_style(segment_text, seg.style, base_size));
        }

        last_end = seg_end.min(line.len());
    }

    if last_end < line.len() {
        let marker = &line[last_end..];
        if !marker.is_empty() {
            let color = if show_markers { shared::MARKER_GRAY } else { theme.background };
            runs.push(TextRun::new(marker, 0, color, base_size));
        }
    }

    runs
}
