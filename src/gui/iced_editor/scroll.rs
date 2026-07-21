//! Автоскролл курсора в видимую зону.
//!
//! При редактировании/навигации курсор может уйти за границу видимой
//! области (вверх или вниз). Этот модуль корректирует `scroll_y` так,
//! чтобы курсор оставался в видимом диапазоне `[0, viewport_height]`.
//!
//! ## Логика
//! 1. Вычислить Y-позицию и высоту строки курсора через `ShapedDocument`.
//! 2. Если `cursor_y < scroll_y` → `scroll_y = cursor_y` (скролл вверх).
//! 3. Если `cursor_y + line_height > scroll_y + viewport_height` →
//!    `scroll_y = cursor_y + line_height - viewport_height` (скролл вниз).

use crate::editor::render::ShapedDocument;

/// Откорректировать `scroll_y` так, чтобы строка с курсором была видна.
///
/// Возвращает новое значение `scroll_y`. Если курсор уже виден — вернёт
/// текущее `scroll_y` без изменений.
pub fn ensure_cursor_visible(
    scroll_y: f32,
    viewport_height: f32,
    shaped: &ShapedDocument,
    cursor_line: usize,
) -> f32 {
    if viewport_height <= 0.0 {
        return scroll_y;
    }

    // Вычисляем Y-позицию верхней границы строки курсора.
    let cursor_y = layout_line_y(shaped, cursor_line);
    let line_h = shaped.line_height(cursor_line);

    if cursor_y < scroll_y {
        // Курсор выше видимой области
        cursor_y
    } else if cursor_y + line_h > scroll_y + viewport_height {
        // Курсор ниже видимой области
        cursor_y + line_h - viewport_height
    } else {
        // Уже виден
        scroll_y
    }
}

/// Y-позиция (line_top) i-й строки.
fn layout_line_y(shaped: &ShapedDocument, line: usize) -> f32 {
    for run in shaped.buffer.layout_runs() {
        if run.line_i == line {
            return run.line_top;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::cache::DocumentCache;
    use crate::editor::render;
    use crate::editor::state::EditMode;
    use crate::editor::theme::EditorTheme;
    use crate::editor::font;

    // ------------------------------------------------------------------
    // helpers
    // ------------------------------------------------------------------

    fn shaped_doc(text: &str, vp_height: f32) -> ShapedDocument {
        font::init();
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics));
        let cache = DocumentCache::default();
        let theme = EditorTheme::default();
        render::build(
            &mut doc,
            text,
            &cache,
            EditMode::LivePreview,
            0,
            &theme,
            14.0,
            24.0,
            0.0,
            Some(vp_height),
        );
        doc
    }

    // ------------------------------------------------------------------
    // ensure_cursor_visible
    // ------------------------------------------------------------------

    #[test]
    fn cursor_already_visible_no_change() {
        let doc = shaped_doc("hello", 200.0);
        let new_scroll = ensure_cursor_visible(0.0, 200.0, &doc, 0);
        assert_eq!(new_scroll, 0.0);
    }

    #[test]
    fn cursor_above_viewport() {
        let doc = shaped_doc("hello\nworld", 200.0);
        // scroll_y = 50, cursor at line 0 (y=0) → cursor is above viewport
        let new_scroll = ensure_cursor_visible(50.0, 100.0, &doc, 0);
        assert!(new_scroll < 50.0, "should scroll up, got {new_scroll}");
        assert_eq!(new_scroll, 0.0); // cursor_y = 0
    }

    #[test]
    fn cursor_below_viewport() {
        let doc = shaped_doc("hello\nworld", 200.0);
        // line 1 has some y > 0; scroll_y=0, vp=small → cursor below
        let h = doc.line_height(1);
        let new_scroll = ensure_cursor_visible(0.0, h - 1.0, &doc, 1);
        // should scroll down so cursor is visible
        assert!(new_scroll > 0.0, "should scroll down, got {new_scroll}");
    }

    #[test]
    fn zero_viewport_no_change() {
        let doc = shaped_doc("hello", 200.0);
        let new_scroll = ensure_cursor_visible(10.0, 0.0, &doc, 0);
        assert_eq!(new_scroll, 10.0); // unchanged
    }

    #[test]
    fn negative_viewport_no_change() {
        let doc = shaped_doc("hello", 200.0);
        let new_scroll = ensure_cursor_visible(10.0, -1.0, &doc, 0);
        assert_eq!(new_scroll, 10.0);
    }

    #[test]
    fn cursor_below_viewport_triggers_scroll() {
        let doc = shaped_doc("a\nb\nc\nline four", 200.0);
        // scroll_y = 0, viewport smaller than cursor's Y → must scroll down
        let cursor_line = 3; // last line
        let cursor_y = doc.buffer.layout_runs().nth(cursor_line).map(|r| r.line_top).unwrap_or(0.0);
        let vp = cursor_y * 0.8; // viewport ends before cursor
        let new_scroll = ensure_cursor_visible(0.0, vp, &doc, cursor_line);
        assert!(
            new_scroll > 0.0,
            "should scroll down (scroll_y=0, vp={vp}, cursor_y={cursor_y}, got new_scroll={new_scroll})"
        );
    }

    // ------------------------------------------------------------------
    // layout_line_y
    // ------------------------------------------------------------------

    #[test]
    fn line_y_zero() {
        let doc = shaped_doc("hello", 200.0);
        let y = layout_line_y(&doc, 0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn line_y_invalid_line() {
        let doc = shaped_doc("hello", 200.0);
        let y = layout_line_y(&doc, 99);
        assert_eq!(y, 0.0);
    }
}
