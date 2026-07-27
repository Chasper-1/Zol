use super::shared;
use super::style::text_run_for_style;
use crate::cache::MarkupCache;
use crate::layout::compensation::LineRunsResult;
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
    compute_line_runs_inner(
        line,
        line_start,
        line_index,
        line_cache,
        base_size,
        heading_size,
        show_markers,
        reveal,
        theme,
        &mut None,
    )
}

/// Разобрать строку + вернуть мета-информацию (скрытые диапазоны).
#[allow(clippy::too_many_arguments)]
pub fn compute_line_runs_with_meta(
    line: &str,
    line_start: usize,
    line_index: usize,
    line_cache: Option<&MarkupCache>,
    base_size: f32,
    heading_size: f32,
    show_markers: bool,
    reveal: Option<&RevealCtx>,
    theme: &EditorTheme,
) -> LineRunsResult {
    let mut hidden_ranges = Vec::new();
    let runs = compute_line_runs_inner(
        line,
        line_start,
        line_index,
        line_cache,
        base_size,
        heading_size,
        show_markers,
        reveal,
        theme,
        &mut Some(&mut hidden_ranges),
    );
    LineRunsResult {
        runs,
        hidden_ranges,
    }
}

/// Внутренняя реализация: если `hidden_out` — Some(&mut Vec), заполняет
/// его диапазонами скрытых маркеров `[start, end)`.
#[allow(clippy::too_many_arguments)]
fn compute_line_runs_inner(
    line: &str,
    line_start: usize,
    line_index: usize,
    line_cache: Option<&MarkupCache>,
    base_size: f32,
    heading_size: f32,
    show_markers: bool,
    reveal: Option<&RevealCtx>,
    theme: &EditorTheme,
    hidden_out: &mut Option<&mut Vec<(usize, usize)>>,
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
                    } else if let Some(hr) = hidden_out {
                        // Маркеры заголовка скрыты
                        hr.push((0, marker_end));
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

    for (i, seg) in cache.segments.iter().enumerate() {
        let seg_start = seg.raw_start.saturating_sub(line_start);
        let seg_end = seg.raw_end.saturating_sub(line_start);

        // ── Текст между сегментами (маркеры / plain text) ──────
        if seg_start > last_end && seg_start <= line.len() {
            let between = &line[last_end..seg_start];
            if !between.is_empty() {
                // Source mode: ВСЁ между сегментами — маркеры (показываем всегда)
                if show_markers {
                    runs.push(TextRun::new(between, 0, shared::MARKER_GRAY, base_size));
                } else {
                    // Live Preview: определяем, маркер это или нет
                    let mut pushed = false;

                    // Проверка 1: открывающий маркер для ТЕКУЩЕГО сегмента
                    let is_open_marker =
                        seg.left_marker_len > 0 && between.len() <= seg.left_marker_len;

                    if is_open_marker {
                        let show = segment_is_revealed(seg, line_index, ctx);
                        if show {
                            runs.push(TextRun::new(between, 0, shared::MARKER_GRAY, base_size));
                        } else if let Some(hr) = hidden_out {
                            hr.push((last_end, seg_start));
                        }
                        pushed = true;
                    }

                    // Проверка 2: закрывающий маркер для ПРЕДЫДУЩЕГО сегмента
                    if !pushed && i > 0 {
                        let prev = &cache.segments[i - 1];
                        let is_close_marker =
                            prev.left_marker_len > 0 && between.len() <= prev.left_marker_len;

                        if is_close_marker {
                            let show = segment_is_revealed(prev, line_index, ctx);
                            if show {
                                runs.push(TextRun::new(between, 0, shared::MARKER_GRAY, base_size));
                            } else if let Some(hr) = hidden_out {
                                hr.push((last_end, seg_start));
                            }
                            pushed = true;
                        }
                    }

                    // Не маркер или скрытый маркер
                    if !pushed {
                        if let Some(hr) = hidden_out {
                            hr.push((last_end, seg_start));
                        }
                    }
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
            } else if let Some(hr) = hidden_out {
                hr.push((last_end, line.len()));
            }
        }
    }

    runs
}
