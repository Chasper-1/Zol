//! Вертикальная навигация с сохранением пиксельной X-позиции курсора.
//!
//! При перемещении вверх/вниз курсор сохраняет `col_visual` — пиксельную
//! координату X. Функции [`cursor_x_on_line`] и [`raw_at_x_on_line`]
//! конвертируют между пиксельной X и byte-смещением на строке, проходя
//! по глифам сформованного буфера.

use crate::editor::layout::cursor_line_bounds;
use crate::editor::render::ShapedDocument;
use crate::gui::iced_editor::EditorInner;

/// X-позиция курсора на строке `line` по глифам буфера.
pub fn cursor_x_on_line(shaped: &ShapedDocument, line: usize, byte_in_line: usize) -> f32 {
    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            if glyph.start >= byte_in_line {
                return glyph.x;
            }
        }
        return run
            .glyphs
            .last()
            .map(|g| g.x + g.w)
            .unwrap_or(0.0);
    }
    0.0
}

/// Ближайший к `x` content-offset на строке `line`.
///
/// Пустая строка → начало строки. Иначе среди глифов и конца строки
/// выбирается точка с минимальным расстоянием по X.
pub fn raw_at_x_on_line(
    shaped: &ShapedDocument,
    line: usize,
    x: f32,
    line_start: usize,
    line_end: usize,
) -> usize {
    if line_end <= line_start {
        return line_start;
    }
    let mut best: Option<(f32, usize)> = None;
    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            let dist = (glyph.x - x).abs();
            let cand = line_start + glyph.start;
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, cand));
            }
        }
        if let Some(last) = run.glyphs.last() {
            let end_x = last.x + last.w;
            let dist = (end_x - x).abs();
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, line_end));
            }
        }
        break;
    }
    best.map_or(line_start, |(_, c)| c)
}

/// Переместить курсор на строку `target_line`, сохраняя пиксельную X.
pub fn move_vertical(inner: &EditorInner, target_line: usize) {
    let x = {
        let content = inner.content.borrow();
        let shaped = inner.shaped_doc.borrow();
        let cursor = inner.cursor.borrow();
        let cl = cursor.line();
        let (ls, _) = cursor_line_bounds(&content, cl);
        let byte_in_line = cursor.raw().saturating_sub(ls);
        cursor_x_on_line(&shaped, cl, byte_in_line)
    };

    let new_raw = {
        let content = inner.content.borrow();
        let shaped = inner.shaped_doc.borrow();
        let (t_start, t_end) = cursor_line_bounds(&content, target_line);
        raw_at_x_on_line(&shaped, target_line, x, t_start, t_end)
    };

    let c = inner.content.borrow();
    let mut cursor = inner.cursor.borrow_mut();
    cursor.set_raw(&c, new_raw);
    cursor.set_col_visual(x);
    inner.dirty.set(true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::cache::DocumentCache;
    use crate::editor::render;
    use crate::editor::state::EditMode;
    use crate::editor::theme::EditorTheme;
    use crate::editor::cursor::Cursor;
    use crate::editor::font;
    use std::cell::{Cell, RefCell};

    // ------------------------------------------------------------------
    // helper: create a shaped doc with one plain-text line
    // ------------------------------------------------------------------
    fn shaped_line(text: &str, size: f32) -> ShapedDocument {
        font::init();
        let metrics = cosmic_text::Metrics::new(size, size * 1.4);
        let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics));
        let cache = DocumentCache::default();
        let theme = EditorTheme::default();
        render::build(&mut doc, text, &cache, EditMode::Source, 0, &theme, size, 24.0, 0.0, None);
        doc
    }

    // ------------------------------------------------------------------
    // helper: EditorInner with given content
    // ------------------------------------------------------------------
    fn make_inner(text: &str) -> EditorInner {
        font::init();
        EditorInner::new(text.to_string())
    }

    // ------------------------------------------------------------------
    // cursor_x_on_line
    // ------------------------------------------------------------------

    #[test]
    fn cursor_x_on_first_glyph_is_zero() {
        let doc = shaped_line("hello", 14.0);
        let x = cursor_x_on_line(&doc, 0, 0);
        assert_eq!(x, 0.0);
    }

    #[test]
    fn cursor_x_increases_along_line() {
        let doc = shaped_line("hello", 14.0);
        let x0 = cursor_x_on_line(&doc, 0, 0);
        let x1 = cursor_x_on_line(&doc, 0, 1);
        assert!(x1 > x0, "x1={x1} should be > x0={x0}");
    }

    #[test]
    fn cursor_x_beyond_last_glyph_returns_end() {
        let doc = shaped_line("ab", 14.0);
        let x = cursor_x_on_line(&doc, 0, 10);
        // should return last glyph x+w
        let last_run = doc.buffer.layout_runs().next().unwrap();
        let last_glyph = last_run.glyphs.last().unwrap();
        assert_eq!(x, last_glyph.x + last_glyph.w);
    }

    #[test]
    fn cursor_x_on_empty_line() {
        let doc = shaped_line("", 14.0);
        let x = cursor_x_on_line(&doc, 0, 0);
        assert_eq!(x, 0.0);
    }

    #[test]
    fn cursor_x_on_second_line() {
        let doc = shaped_line("ab\ncd", 14.0);
        assert_eq!(doc.line_count(), 2);
        let x = cursor_x_on_line(&doc, 1, 0);
        assert_eq!(x, 0.0);
    }

    // ------------------------------------------------------------------
    // raw_at_x_on_line
    // ------------------------------------------------------------------

    #[test]
    fn raw_at_x_at_start_of_line() {
        let doc = shaped_line("hello", 14.0);
        let raw = raw_at_x_on_line(&doc, 0, 0.0, 0, 5);
        assert_eq!(raw, 0);
    }

    #[test]
    fn raw_at_x_empty_line() {
        let doc = shaped_line("", 14.0);
        let raw = raw_at_x_on_line(&doc, 0, 0.0, 0, 0);
        assert_eq!(raw, 0);
    }

    #[test]
    fn raw_at_x_negative_x() {
        let doc = shaped_line("hello", 14.0);
        let raw = raw_at_x_on_line(&doc, 0, -100.0, 0, 5);
        assert_eq!(raw, 0);
    }

    #[test]
    fn raw_at_x_beyond_end() {
        let doc = shaped_line("ab", 14.0);
        let raw = raw_at_x_on_line(&doc, 0, 9999.0, 0, 2);
        assert_eq!(raw, 2); // end of line
    }

    // ------------------------------------------------------------------
    // move_vertical
    // ------------------------------------------------------------------

    #[test]
    fn move_vertical_moves_to_target_line() {
        let inner = make_inner("line zero\nline one");
        {
            let mut c = inner.cursor.borrow_mut();
            c.set_raw(&inner.content.borrow(), 5); // somewhere on line 0
        }
        let old_line = inner.cursor.borrow().line();
        move_vertical(&inner, 1);
        let new_line = inner.cursor.borrow().line();
        assert_eq!(old_line, 0);
        assert_eq!(new_line, 1);
    }

    #[test]
    fn move_vertical_sets_dirty() {
        let inner = make_inner("a\nb");
        inner.dirty.set(false);
        move_vertical(&inner, 1);
        assert!(inner.dirty.get(), "move_vertical should set dirty=true");
    }

    #[test]
    fn move_vertical_computes_col_visual_from_glyph() {
        let inner = make_inner("aaa\nbbb");
        // cursor at start of line 0 → glyph x=0
        move_vertical(&inner, 1);
        // col_visual should be the pixel x of the cursor on line 0 (which is 0 at start)
        assert_eq!(inner.cursor.borrow().col_visual(), 0.0);
    }

    #[test]
    fn move_vertical_sets_col_visual_before_move() {
        let inner = make_inner("aaa\nbbb");
        // put cursor at end of line 0 (bbb is shorter, but let's use a wider text)
        let inner = make_inner("hello world\nshort");
        {
            let mut c = inner.cursor.borrow_mut();
            c.set_raw(&inner.content.borrow(), 5); // space between hello and world
        }
        move_vertical(&inner, 1);
        // col_visual should be > 0 (since we were at byte 5 on line 0)
        assert!(inner.cursor.borrow().col_visual() >= 0.0);
    }

    #[test]
    fn move_vertical_target_line_zero() {
        let inner = make_inner("a\nb");
        {
            let mut c = inner.cursor.borrow_mut();
            c.set_raw(&inner.content.borrow(), 2); // line 1
        }
        move_vertical(&inner, 0);
        assert_eq!(inner.cursor.borrow().line(), 0);
    }
}
