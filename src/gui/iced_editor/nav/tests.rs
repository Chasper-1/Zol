use super::*;
use crate::editor::cache::DocumentCache;
use crate::editor::font;
use crate::editor::render;
use crate::editor::render::ShapedDocument;
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;
use crate::gui::iced_editor::EditorInner;

// ------------------------------------------------------------------
// helper: create a shaped doc with one plain-text line
// ------------------------------------------------------------------
fn shaped_line(text: &str, size: f32) -> ShapedDocument {
    font::init();
    let metrics = cosmic_text::Metrics::new(size, size * 1.4);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    render::build(
        &mut doc, text, &cache, EditMode::Source, 0, &theme, size, 24.0, 0.0, None,
    );
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
#[test]
fn cursor_x_roundtrip_bold_cyrillic() {
    let text = "**текст**";
    let doc = shaped_line(text, 14.0);
    let run = doc.buffer.layout_runs().next().unwrap();
    let glyphs: Vec<_> = run.glyphs.iter().map(|g| (g.start, g.x, g.w)).collect();

    let mut boundaries = vec![0usize];
    for &(start, _, _) in &glyphs {
        if start != *boundaries.last().unwrap() {
            boundaries.push(start);
        }
    }
    let line_end = text.len();
    if *boundaries.last().unwrap() != line_end {
        boundaries.push(line_end);
    }

    for &byte_in_line in &boundaries {
        let x = cursor_x_on_line(&doc, 0, byte_in_line);
        let recovered = raw_at_x_on_line(&doc, 0, x, 0, line_end);
        assert!(
            recovered == byte_in_line,
            "byte_in_line={}: x={} recovered={}. glyphs={:?}",
            byte_in_line,
            x,
            recovered,
            glyphs
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

/// Helper: установить курсор на конкретный байт.
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
