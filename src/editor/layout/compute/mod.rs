//! Чистая раскладка строки: сегменты → [`TextRun`]ы.
//!
//! Никаких зависимостей от GUI-фреймворков.
//! Результат можно скормить адаптеру в `render/mod.rs`.

use super::types::TextRun;
use crate::editor::cache::MarkupCache;
use crate::editor::markup::segment::{
    STYLE_BOLD, STYLE_CODE, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA,
    STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_SUBSCRIPT,
    STYLE_SUPERSCRIPT, STYLE_UNDERLINE,
};
use crate::editor::theme::color::Rgba;
use crate::editor::theme::EditorTheme;
use crate::editor::utils::line_utils;

/// Разобрать строку на стилизованные фрагменты.
///
/// Возвращает список `TextRun` с текстом, цветом, размером и флагами стиля.
/// Маркерные символы (если `show_markers = true`) выделяются серым.
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
    // Заголовок (#) — маркер всегда в buffer'е для hit-testing
    if let Some(stripped) = line.strip_prefix("# ") {
        let mut runs = Vec::new();
        let color = if show_markers { shared::MARKER_GRAY } else { theme.background };
        runs.push(TextRun::new("# ", 0, color, heading_size));
        runs.push(TextRun::new(stripped, 0, shared::TEXT_WHITE, heading_size));
        return runs;
    }

    // Нет кэша или нет сегментов — вся строка plain (цвет из темы)
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

        // Маркер-текст между сегментами (например, "**")
        // Всегда в buffer'е для корректного hit-testing — в Preview/LivePreview
        // красим цветом фона (невидимо), в Source — серым.
        if seg_start > last_end && seg_start <= line.len() {
            let marker = &line[last_end..seg_start];
            if !marker.is_empty() {
                let color = if show_markers { shared::MARKER_GRAY } else { theme.background };
                runs.push(TextRun::new(marker, 0, color, base_size));
            }
        }

        // Сегмент
        if seg_start < line.len() {
            let end = seg_end.min(line.len());
            let segment_text = &line[seg_start..end];
            runs.push(text_run_for_style(segment_text, seg.style, base_size, heading_size));
        }

        last_end = seg_end.min(line.len());
    }

    // Остаток строки после последнего сегмента (маркеры)
    if last_end < line.len() {
        let marker = &line[last_end..];
        if !marker.is_empty() {
            let color = if show_markers { shared::MARKER_GRAY } else { theme.background };
            runs.push(TextRun::new(marker, 0, color, base_size));
        }
    }

    runs
}

/// Создать `TextRun` по битовым флагам стиля.
fn text_run_for_style(
    text: &str,
    style: u32,
    base_size: f32,
    _heading_size: f32,
) -> TextRun {
    // Цвет по умолчанию
    let mut color = shared::TEXT_DEFAULT;
    let mut size = base_size;
    let mut family: Option<&str> = None;

    if style & STYLE_BOLD != 0 {
        color = Rgba::new(1.0, 0.392, 0.392); // #FF6464
    }
    if style & STYLE_ITALIC != 0 {
        color = Rgba::new(0.392, 0.784, 1.0); // #64C8FF
    }
    if style & STYLE_CODE != 0 {
        color = Rgba::new(0.784, 0.784, 0.784);
        family = Some("monospace");
    }
    if style & STYLE_UNDERLINE != 0 {
        // цвет не меняем, флаг уйдёт в adapter
    }
    if style & STYLE_HIGHLIGHT != 0 {
        // фон — не храним в TextRun (будет в adapter)
    }
    if style & STYLE_INSERTION != 0 {
        color = Rgba::new(0.392, 1.0, 0.392);
    }
    if style & STYLE_DELETION != 0 {
        color = Rgba::new(1.0, 0.314, 0.314);
    }
    if style & STYLE_COMMENT != 0 {
        color = Rgba::new(0.549, 0.549, 0.549);
    }
    if style & STYLE_STRIKETHROUGH != 0 {
        // цвет не меняем
    }
    if style & STYLE_SUPERSCRIPT != 0 {
        size = base_size * 0.7;
        color = Rgba::new(0.588, 1.0, 0.588);
    }
    if style & STYLE_SUBSCRIPT != 0 {
        size = base_size * 0.7;
        color = Rgba::new(1.0, 0.784, 0.392);
    }
    if style & STYLE_FORMULA != 0 {
        color = Rgba::new(0.314, 0.863, 0.471);
        family = Some("monospace");
    }
    if style & STYLE_DISPLAY_FORMULA != 0 {
        size = base_size * 1.3;
        color = Rgba::new(0.314, 0.863, 0.471);
        family = Some("monospace");
    }

    let mut run = TextRun::new(text, style, color, size);
    if let Some(f) = family {
        run = run.with_font(f);
    }
    run
}

/// Границы строки в байтах для позиционирования курсора.
pub fn cursor_line_bounds(content: &str, line: usize) -> (usize, usize) {
    line_utils::line_bounds(content, line)
        .map(|b| (b.start, b.end))
        .unwrap_or((0, 0))
}

/// Общие константы для цветов раскладки.
mod shared {
    use crate::editor::theme::color::Rgba;

    pub const TEXT_DEFAULT: Rgba = Rgba::new(0.863, 0.863, 0.863); // #DCDCDC
    pub const TEXT_WHITE: Rgba = Rgba::new(1.0, 1.0, 1.0);
    pub const MARKER_GRAY: Rgba = Rgba::new(0.392, 0.392, 0.392); // #646464
}


#[cfg(test)]
mod tests;
