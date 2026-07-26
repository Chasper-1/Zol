use super::shared;
use super::style::text_run_for_style;
use crate::cache::MarkupCache;
use crate::layout::reveal::RevealState;
use crate::layout::types::TextRun;
use crate::theme::EditorTheme;

/// Разобрать строку на стилизованные фрагменты.
///
/// Если `show_markers == false` и сегмент **не** раскрыт через `revealed`,
/// текст его маркеров **физически исключается** из результата (не занимает места).
/// Если раскрыт — маркеры отображаются серым как обычно.
#[allow(clippy::too_many_arguments)]
pub fn compute_line_runs(
    line: &str,
    line_start: usize,
    line_index: usize,
    line_cache: Option<&MarkupCache>,
    base_size: f32,
    heading_size: f32,
    show_markers: bool,
    revealed: Option<&RevealState>,
    theme: &EditorTheme,
) -> Vec<TextRun> {
    // ─── Заголовок #N# ────────────────────────────────────────────────
    if let Some(rest) = line.strip_prefix('#') {
        if let Some(level_end) = rest.find('#') {
            if level_end > 0 {
                let level_str = &rest[..level_end];
                if level_str.parse::<u32>().is_ok() {
                    let marker_end = level_end + 2;
                    let content = line[marker_end..].trim_start();
                    let mut runs = Vec::new();

                    if show_markers {
                        let marker_color = shared::MARKER_GRAY;
                        let marker_text = &line[..marker_end];
                        runs.push(TextRun::new(marker_text, 0, marker_color, heading_size));
                    }
                    // Контент всегда показываем
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
                // Это может быть как маркер, так и просто plain text между сегментами.
                // Показываем только если show_markers или сегмент раскрыт.
                // Определяем, относится ли этот кусок к маркеру текущего сегмента.
                let left_marker_len = seg.left_marker_len;
                let is_marker = left_marker_len > 0 && between.len() <= left_marker_len;

                let revealed_for_seg = revealed
                    .map(|r| r.is_revealed(line_index, seg.raw_start))
                    .unwrap_or(false);

                if show_markers || (is_marker && revealed_for_seg) {
                    let color = if show_markers || revealed_for_seg {
                        shared::MARKER_GRAY
                    } else {
                        theme.background
                    };
                    runs.push(TextRun::new(between, 0, color, base_size));
                }
                // else: маркер скрыт — не включаем в runs
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
            // Остаток всегда является маркером (закрывающим).
            // Показываем только если show_markers.
            let is_closing_marker = cache
                .segments
                .last()
                .map(|seg| seg.right_marker_len > 0)
                .unwrap_or(false);

            let last_seg_raw = cache.segments.last().map(|s| s.raw_start).unwrap_or(0);
            let revealed_for_last = revealed
                .map(|r| r.is_revealed(line_index, last_seg_raw))
                .unwrap_or(false);

            if show_markers || (is_closing_marker && revealed_for_last) {
                let color = if show_markers || revealed_for_last {
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
