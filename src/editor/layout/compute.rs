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
    // Заголовок (#)
    if let Some(stripped) = line.strip_prefix("# ") {
        let mut runs = Vec::new();
        if show_markers {
            runs.push(TextRun::new("# ", 0, shared::MARKER_GRAY, heading_size));
        }
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
        if show_markers && seg_start > last_end && seg_start <= line.len() {
            let marker = &line[last_end..seg_start];
            if !marker.is_empty() {
                runs.push(TextRun::new(marker, 0, shared::MARKER_GRAY, base_size));
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
    if show_markers && last_end < line.len() {
        let marker = &line[last_end..];
        if !marker.is_empty() {
            runs.push(TextRun::new(marker, 0, shared::MARKER_GRAY, base_size));
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
mod tests {
    use super::*;
    use crate::editor::cache::MarkupCache;
    use crate::editor::markup::segment::{Segment, STYLE_BOLD, STYLE_CODE, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA, STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_SUBSCRIPT, STYLE_SUPERSCRIPT, STYLE_UNDERLINE};

    fn cache(segments: Vec<Segment>) -> MarkupCache {
        MarkupCache { segments }
    }

    fn seg(style: u32, raw_start: usize, raw_end: usize) -> Segment {
        Segment {
            text: String::new(),
            style,
            left_marker_len: 0,
            right_marker_len: 0,
            raw_start,
            raw_end,
        }
    }

    #[test]
    fn plain_line_no_cache() {
        let runs = compute_line_runs("hello", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "hello");
        assert_eq!(runs[0].size, 14.0);
    }

    #[test]
    fn plain_line_with_cache_empty_segments() {
        let runs = compute_line_runs("hello", 0, Some(&cache(vec![])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "hello");
    }

    #[test]
    fn empty_line() {
        let runs = compute_line_runs("", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "");
    }

    #[test]
    fn empty_line_with_markers() {
        let runs = compute_line_runs("", 0, None, 14.0, 22.0, true, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn heading_no_markers() {
        let runs = compute_line_runs("# hi", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "hi");
        assert_eq!(runs[0].size, 22.0);
    }

    #[test]
    fn heading_with_markers() {
        let runs = compute_line_runs("# hi", 0, None, 14.0, 22.0, true, &EditorTheme::default());
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text, "# ");
        assert_eq!(runs[0].size, 22.0);
        assert_eq!(runs[1].text, "hi");
    }

    #[test]
    fn heading_empty_after_prefix() {
        let runs = compute_line_runs("# ", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "");
    }

    #[test]
    fn not_heading_without_space() {
        // "#no" — не заголовок (нет пробела после #)
        let runs = compute_line_runs("#no", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "#no");
        assert_eq!(runs[0].size, 14.0);
    }

    #[test]
    fn bold_segment_with_markers() {
        // "a **b** c" — байты: a=0, ' '=1, '*=2, '*=3, b=4, '*=5, '*=6, ' '=7, c=8
        let seg = seg(STYLE_BOLD, 2, 8);
        let runs = compute_line_runs(
            "a **b** c",
            0,
            Some(&cache(vec![seg])),
            14.0,
            22.0,
            true,
            &EditorTheme::default(),
        );
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "a "); // маркер (plain text) до сегмента
        assert_eq!(runs[1].text, "**b** "); // сегмент: **b** + пробел (raw 2..8)
        assert_eq!(runs[2].text, "c"); // маркер после сегмента
    }

    #[test]
    fn bold_segment_no_markers() {
        let seg = seg(STYLE_BOLD, 2, 8);
        let runs = compute_line_runs(
            "a **b** c",
            0,
            Some(&cache(vec![seg])),
            14.0,
            22.0,
            false,
            &EditorTheme::default(),
        );
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "**b** "); // только сегмент, маркеры скрыты
    }

    #[test]
    fn bold_segment_color() {
        let seg = seg(STYLE_BOLD, 0, 4);
        let runs = compute_line_runs("bold", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert!(runs[0].color.r > 0.8, "bold should be reddish: {:?}", runs[0].color);
    }

    #[test]
    fn italic_segment_color() {
        let seg = seg(STYLE_ITALIC, 0, 6);
        let runs = compute_line_runs("italic", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.b > 0.8, "italic should be bluish: {:?}", runs[0].color);
    }

    #[test]
    fn code_segment() {
        let seg = seg(STYLE_CODE, 0, 4);
        let runs = compute_line_runs("code", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].font_family.as_deref(), Some("monospace"));
    }

    #[test]
    fn insertion_color() {
        let seg = seg(STYLE_INSERTION, 0, 4);
        let runs = compute_line_runs("inst", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.g > 0.8, "insertion should be greenish: {:?}", runs[0].color);
    }

    #[test]
    fn deletion_color() {
        let seg = seg(STYLE_DELETION, 0, 4);
        let runs = compute_line_runs("del", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.r > 0.8, "deletion should be reddish: {:?}", runs[0].color);
    }

    #[test]
    fn comment_color() {
        let seg = seg(STYLE_COMMENT, 0, 7);
        let runs = compute_line_runs("comment", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.r < 0.6, "comment should be gray: {:?}", runs[0].color);
    }

    #[test]
    fn superscript_smaller_size() {
        let seg = seg(STYLE_SUPERSCRIPT, 0, 5);
        let runs = compute_line_runs("super", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].size, 14.0 * 0.7);
    }

    #[test]
    fn subscript_smaller_size() {
        let seg = seg(STYLE_SUBSCRIPT, 0, 5);
        let runs = compute_line_runs("sub", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].size, 14.0 * 0.7);
    }

    #[test]
    fn formula_monospace() {
        let seg = seg(STYLE_FORMULA, 0, 4);
        let runs = compute_line_runs("form", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].font_family.as_deref(), Some("monospace"));
    }

    #[test]
    fn display_formula_larger() {
        let seg = seg(STYLE_DISPLAY_FORMULA, 0, 4);
        let runs = compute_line_runs("disp", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].size, 14.0 * 1.3);
        assert_eq!(runs[0].font_family.as_deref(), Some("monospace"));
    }

    #[test]
    fn strikethrough_does_not_change_color() {
        let seg = seg(STYLE_STRIKETHROUGH, 0, 5);
        let runs = compute_line_runs("strike", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        // strikethrough doesn't change the color, so it should stay default
        let default_color = shared::TEXT_DEFAULT;
        assert_eq!(runs[0].color.r, default_color.r);
        assert_eq!(runs[0].color.g, default_color.g);
        assert_eq!(runs[0].color.b, default_color.b);
    }

    #[test]
    fn highlight_does_not_change_color() {
        let seg = seg(STYLE_HIGHLIGHT, 0, 5);
        let runs = compute_line_runs("high", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        let default_color = shared::TEXT_DEFAULT;
        assert_eq!(runs[0].color.r, default_color.r);
        assert_eq!(runs[0].color.g, default_color.g);
        assert_eq!(runs[0].color.b, default_color.b);
    }

    #[test]
    fn multiple_segments_on_line() {
        // line = "hello **world** again"  (21 bytes)
        // seg covers "**world**" (including both marker stars): raw_start=6, raw_end=15
        // bytes: h=0,e=1,l=2,l=3,o=4,' '=5,'*'=6,'*'=7,w=8,o=9,r=10,l=11,d=12,'*'=13,'*'=14,' '=15,...
        let seg = seg(STYLE_BOLD, 6, 15);
        let runs = compute_line_runs("hello **world** again", 0, Some(&cache(vec![seg])), 14.0, 22.0, true, &EditorTheme::default());
        // 3 runs: marker "hello ", bold "**world**", marker " again"
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "hello ");
        assert_eq!(runs[1].text, "**world**");
        assert!(runs[1].style_flags & STYLE_BOLD != 0);
        assert_eq!(runs[2].text, " again");
    }

    #[test]
    fn segments_back_to_back() {
        // "abc**bold** ~~strike~~ end"
        // bytes: a=0,b=1,c=2,*=3,*=4,b=5,o=6,l=7,d=8,*=9,*=10,' '=11,'~'=12,'~'=13,s=14,t=15,r=16,i=17,k=18,e=19,'~'=20,'~'=21,' '=22,e=23,n=24,d=25
        // seg1 bold: raw 3..11 = "**bold**"
        // seg2 strikethrough: raw 11..22 = " ~~strike~~" (includes leading space to separate)
        let seg1 = seg(STYLE_BOLD, 3, 11);
        let seg2 = seg(STYLE_STRIKETHROUGH, 11, 22);
        let runs = compute_line_runs("abc**bold** ~~strike~~ end", 0, Some(&cache(vec![seg1, seg2])), 14.0, 22.0, true, &EditorTheme::default());
        // marker "abc", bold "**bold**", strikethrough " ~~strike~~", marker " end"
        assert_eq!(runs.len(), 4);
        assert_eq!(runs[0].text, "abc");
        assert_eq!(runs[1].text, "**bold**");
        assert!(runs[1].style_flags & STYLE_BOLD != 0);
        assert_eq!(runs[2].text, " ~~strike~~");
        assert!(runs[2].style_flags & STYLE_STRIKETHROUGH != 0);
        assert_eq!(runs[3].text, " end");
    }

    #[test]
    fn segment_at_line_start() {
        let seg = seg(STYLE_BOLD, 0, 4);
        let runs = compute_line_runs("bold", 0, Some(&cache(vec![seg])), 14.0, 22.0, true, &EditorTheme::default());
        assert_eq!(runs.len(), 1); // нет маркеров до сегмента, если сегмент с начала
        assert_eq!(runs[0].text, "bold");
    }

    #[test]
    fn segment_past_line_end_is_clamped() {
        // seg_end выходит за длину строки
        let seg = seg(STYLE_BOLD, 0, 100);
        let runs = compute_line_runs("short", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "short");
    }

    #[test]
    fn cursor_line_bounds_works() {
        let (start, end) = cursor_line_bounds("abc\ndef\nghi", 1);
        assert_eq!(start, 4);
        assert_eq!(end, 7);
    }

    #[test]
    fn cursor_line_bounds_out_of_range() {
        let (start, end) = cursor_line_bounds("abc", 99);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn cursor_line_bounds_single_line() {
        let (start, end) = cursor_line_bounds("abc", 0);
        assert_eq!(start, 0);
        assert_eq!(end, 3);
    }
}
