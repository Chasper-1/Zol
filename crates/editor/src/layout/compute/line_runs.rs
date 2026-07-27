use super::shared;
use super::style::text_run_for_style;
use crate::cache::MarkupCache;
use crate::layout::reveal::{RevealCtx, segment_is_revealed};
use crate::layout::types::TextRun;
use crate::theme::EditorTheme;

/// Разобрать строку на стилизованные фрагменты.
///
/// Если `show_markers == false` — маркеры скрыты, но автоматически
/// раскрываются, если курсор рядом (inline) / на той же строке (line) /
/// внутри того же блока (block). `reveal` — контекст раскрытия;
/// `None` — маркеры всегда скрыты.
#[allow(clippy::too_many_arguments)]
pub fn compute_line_runs(
    line: &str,
    line_start: usize,
    line_index: usize,
    line_cache: Option<&MarkupCache>,
    base_size: f32,
    heading_size: f32,
    show_markers: bool,
    reveal: Option<&RevealCtx>,
    theme: &EditorTheme,
) -> Vec<TextRun> {
    let ctx = reveal.unwrap_or(RevealCtx::empty());

    // ─── Заголовок #N# ────────────────────────────────────────────────
    if let Some(rest) = line.strip_prefix('#') {
        if let Some(level_end) = rest.find('#') {
            if level_end > 0 {
                let level_str = &rest[..level_end];
                if level_str.parse::<u32>().is_ok() {
                    let marker_end = level_end + 2;
                    let content = line[marker_end..].trim_start();
                    let mut runs = Vec::new();
                    let line_is_active = ctx.cursor_line == Some(line_index);

                    if show_markers || line_is_active {
                        let marker_color = shared::MARKER_GRAY;
                        let marker_text = &line[..marker_end];
                        runs.push(TextRun::new(marker_text, 0, marker_color, heading_size));
                    }
                    runs.push(TextRun::new(content, 0, shared::TEXT_WHITE, heading_size));
                    return runs;
                }
            }
        }
    }

    // ─── Нет кеша или пустой кеш → простой текст ──────────────────────
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

        // ── Текст между сегментами (маркеры / plain text) ──────
        if seg_start > last_end && seg_start <= line.len() {
            let between = &line[last_end..seg_start];
            if !between.is_empty() {
                let left_marker_len = seg.left_marker_len;
                let is_marker = left_marker_len > 0 && between.len() <= left_marker_len;
                let show = segment_is_revealed(seg, line_index, ctx);

                if show_markers || (is_marker && show) {
                    let color = if show_markers || show {
                        shared::MARKER_GRAY
                    } else {
                        theme.background
                    };
                    runs.push(TextRun::new(between, 0, color, base_size));
                }
            }
        }

        // ── Сегмент ──
        if seg_start < line.len() {
            let end = seg_end.min(line.len());
            let segment_text = &line[seg_start..end];
            runs.push(text_run_for_style(segment_text, seg.style, base_size));
        }

        last_end = seg_end.min(line.len());
    }

    // ── Остаток строки после последнего сегмента ──────────────
    if last_end < line.len() {
        let remaining = &line[last_end..];
        if !remaining.is_empty() {
            let show = cache.segments.last().map_or(false, |seg| {
                seg.left_marker_len > 0 && segment_is_revealed(seg, line_index, ctx)
            });

            if show_markers || show {
                let color = if show_markers || show {
                    shared::MARKER_GRAY
                } else {
                    theme.background
                };
                runs.push(TextRun::new(remaining, 0, color, base_size));
            }
        }
    }

    runs
}
