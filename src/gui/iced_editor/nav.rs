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
///
/// Мутирует только позицию курсора, контент не меняется.
pub fn move_vertical(inner: &EditorInner, target_line: usize) {
    // 1. Вычисляем текущую X-позицию курсора на исходной строке
    let x = {
        let doc = inner.doc.borrow();
        let shaped = inner.shaped_doc.borrow();
        let cl = doc.cursor.line();
        let (ls, _) = cursor_line_bounds(&doc.content, cl);
        let byte_in_line = doc.cursor.raw().saturating_sub(ls);
        cursor_x_on_line(&shaped, cl, byte_in_line)
    };

    // 2. Вычисляем новый byte-offset на целевой строке
    let new_raw = {
        let doc = inner.doc.borrow();
        let shaped = inner.shaped_doc.borrow();
        let (t_start, t_end) = cursor_line_bounds(&doc.content, target_line);
        raw_at_x_on_line(&shaped, target_line, x, t_start, t_end)
    };

    // 3. Мутируем курсор (content не меняется — кеш не трогаем).
    //    Клонируем content заранее — RefMut не позволяет split borrows.
    let content = inner.doc.borrow().content.clone();
    {
        let mut doc = inner.doc.borrow_mut();
        doc.cursor.set_raw(&content, new_raw);
        doc.cursor.set_col_visual(x);
        doc.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::cache::DocumentCache;
    use crate::editor::render;
    use crate::editor::state::EditMode;
    use crate::editor::theme::EditorTheme;
    use crate::editor::font;
    use std::cell::RefCell;

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

    /// Сквозная проверка курсора для "**текст**" с кириллицей и маркерами.
    /// Проверяет, что cursor_x_on_line возвращает корректную x для каждой pos
    /// и что raw_at_x_on_line (hit-testing) даёт обратно ту же позицию.
    #[test]
    fn cursor_x_roundtrip_bold_cyrillic() {
        let text = "**текст**";
        let doc = shaped_line(text, 14.0);
        let run = doc.buffer.layout_runs().next().unwrap();
        let glyphs: Vec<_> = run.glyphs.iter().map(|g| (g.start, g.x, g.w)).collect();

        // Для каждой границы между кластерами проверяем roundtrip:
        //   cursor_x_on_line(byte_in_line) → x
        //   raw_at_x_on_line(x) → byte_in_line
        // Должны получить ту же позицию (или соседнюю валидную).
        let mut boundaries = vec![0usize];
        for &(start, _, _) in &glyphs {
            if start != *boundaries.last().unwrap() {
                boundaries.push(start);
            }
        }
        // Добавляем конец строки
        let line_end = text.len();
        if *boundaries.last().unwrap() != line_end {
            boundaries.push(line_end);
        }

        for &byte_in_line in &boundaries {
            let x = cursor_x_on_line(&doc, 0, byte_in_line);
            let recovered = raw_at_x_on_line(&doc, 0, x, 0, line_end);
            // recovered должен быть byte_in_line или соседней границей
            assert!(
                recovered == byte_in_line,
                "byte_in_line={}: x={} recovered={}. glyphs={:?}",
                byte_in_line, x, recovered, glyphs
            );
        }
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
        assert_eq!(raw, 2);
    }

    // ------------------------------------------------------------------
    // move_vertical
    // ------------------------------------------------------------------

    /// Helper: установить курсор на конкретный байт (для тестов).
    fn set_cursor_raw(inner: &EditorInner, raw: usize) {
        let content = inner.doc.borrow().content.clone();
        let mut doc = inner.doc.borrow_mut();
        doc.cursor.set_raw(&content, raw);
    }

    #[test]
    fn move_vertical_moves_to_target_line() {
        let inner = make_inner("line zero\nline one");
        set_cursor_raw(&inner, 5);
        let old_line = inner.doc.borrow().cursor.line();
        move_vertical(&inner, 1);
        let new_line = inner.doc.borrow().cursor.line();
        assert_eq!(old_line, 0);
        assert_eq!(new_line, 1);
    }

    #[test]
    fn move_vertical_sets_dirty() {
        let inner = make_inner("a\nb");
        inner.doc.borrow_mut().dirty = false;
        move_vertical(&inner, 1);
        assert!(inner.doc.borrow().dirty, "move_vertical should set dirty=true");
    }

    #[test]
    fn move_vertical_computes_col_visual_from_glyph() {
        let inner = make_inner("aaa\nbbb");
        move_vertical(&inner, 1);
        assert_eq!(inner.doc.borrow().cursor.col_visual(), 0.0);
    }

    #[test]
    fn move_vertical_sets_col_visual_before_move() {
        let inner = make_inner("hello world\nshort");
        set_cursor_raw(&inner, 5);
        move_vertical(&inner, 1);
        assert!(inner.doc.borrow().cursor.col_visual() >= 0.0);
    }

    #[test]
    fn move_vertical_target_line_zero() {
        let inner = make_inner("a\nb");
        set_cursor_raw(&inner, 2);
        move_vertical(&inner, 0);
        assert_eq!(inner.doc.borrow().cursor.line(), 0);
    }
}
