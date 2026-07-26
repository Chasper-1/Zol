use super::*;
use editor::cache::DocumentCache;
use editor::font;
use editor::render;
use editor::render::ShapedDocument;
use editor::state::EditMode;
use editor::theme::EditorTheme;

// ------------------------------------------------------------------
// helpers
// ------------------------------------------------------------------

fn shaped_doc(text: &str, vp_height: f32) -> ShapedDocument {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
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
        None,
    );
    doc
}

fn shaped_multi(text: &str) -> ShapedDocument {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
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
        Some(1000.0),
        None,
    );
    doc
}

// ═══════════════════════════════════════════════════════════════════
// ensure_cursor_visible
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cursor_already_visible_no_change() {
    let doc = shaped_doc("hello", 200.0);
    let new_scroll = ensure_cursor_visible(0.0, 200.0, &doc, 0);
    assert_eq!(new_scroll, 0.0);
}

#[test]
fn cursor_above_viewport_scrolls_up() {
    let doc = shaped_doc("hello\nworld", 200.0);
    let new_scroll = ensure_cursor_visible(50.0, 100.0, &doc, 0);
    assert!(new_scroll < 50.0, "should scroll up, got {new_scroll}");
    assert_eq!(new_scroll, 0.0);
}

#[test]
fn cursor_below_viewport_scrolls_down() {
    let doc = shaped_doc("hello\nworld", 200.0);
    let h = doc.line_height(1);
    let new_scroll = ensure_cursor_visible(0.0, h - 1.0, &doc, 1);
    assert!(new_scroll > 0.0, "should scroll down, got {new_scroll}");
}

#[test]
fn zero_viewport_no_change() {
    let doc = shaped_doc("hello", 200.0);
    let new_scroll = ensure_cursor_visible(10.0, 0.0, &doc, 0);
    assert_eq!(new_scroll, 10.0);
}

#[test]
fn negative_viewport_no_change() {
    let doc = shaped_doc("hello", 200.0);
    let new_scroll = ensure_cursor_visible(10.0, -1.0, &doc, 0);
    assert_eq!(new_scroll, 10.0);
}

#[test]
fn cursor_on_last_line_triggers_scroll() {
    let doc = shaped_multi("a\nb\nc\nline four");
    let cursor_line = 3;
    let new_scroll = ensure_cursor_visible(0.0, 30.0, &doc, cursor_line);
    assert!(new_scroll > 0.0, "should scroll down for last line");
}

#[test]
fn cursor_on_first_line_no_scroll() {
    let doc = shaped_multi("a\nb\nc\nd\ne\nf");
    let new_scroll = ensure_cursor_visible(50.0, 100.0, &doc, 0);
    assert_eq!(new_scroll, 0.0, "should scroll to top for first line");
}

#[test]
fn cursor_visible_stays() {
    let doc = shaped_multi("a\nb\nc\nd\ne\nf\nghijklmn");
    // Строка 0 (y=0) видна при scroll_y=0, viewport=200
    let result = ensure_cursor_visible(0.0, 200.0, &doc, 0);
    assert_eq!(result, 0.0, "visible cursor should not change scroll");
}

#[test]
fn cursor_on_mid_line_visible_stays() {
    let doc = shaped_multi("a\nb\nc\nd\ne\nf\nghijklmn");
    let _line_y = layout_line_y(&doc, 3);
    // scroll_y=0, line 3 visible in viewport
    let result = ensure_cursor_visible(0.0, 200.0, &doc, 3);
    assert_eq!(result, 0.0);
}

#[test]
fn cursor_slightly_below_viewport_scrolls() {
    let doc = shaped_multi("line0\nline1\nline2\nline3\nline4");
    let cursor_line = 4;
    let scroll_y = 0.0;
    let vp_h = doc.line_height(4) * 3.0; // viewport показывает только 3 строки
    let new_scroll = ensure_cursor_visible(scroll_y, vp_h, &doc, cursor_line);
    assert!(
        new_scroll > scroll_y,
        "should scroll down from {scroll_y} to {new_scroll}"
    );
}

#[test]
fn cursor_barely_above_viewport_scrolls() {
    let doc = shaped_multi("line0\nline1\nline2");
    // scroll_y = 100, cursor на строке 0 (y=0) — выше viewport
    let new_scroll = ensure_cursor_visible(100.0, 50.0, &doc, 0);
    assert_eq!(new_scroll, 0.0, "should scroll to top");
}

#[test]
fn single_line_always_visible() {
    let doc = shaped_doc("single line", 200.0);
    let new_scroll = ensure_cursor_visible(0.0, 200.0, &doc, 0);
    assert_eq!(new_scroll, 0.0);
    let new_scroll2 = ensure_cursor_visible(50.0, 200.0, &doc, 0);
    assert_eq!(new_scroll2, 0.0, "single line at scroll 50 should jump to 0");
}

#[test]
fn cursor_line_out_of_range_returns_line_y() {
    // layout_line_y для несуществующей линии возвращает 0.0
    // ensure_cursor_visible видит cursor_y(0) < scroll_y(10) → scroll down to 0
    let doc = shaped_doc("hello", 200.0);
    let new_scroll = ensure_cursor_visible(10.0, 100.0, &doc, 99);
    assert_eq!(new_scroll, 0.0, "out of range line y=0, scroll should jump to 0");
}

#[test]
fn empty_doc_scroll_unchanged() {
    let doc = shaped_doc("", 200.0);
    let new_scroll = ensure_cursor_visible(0.0, 100.0, &doc, 0);
    assert_eq!(new_scroll, 0.0);
}

// ═══════════════════════════════════════════════════════════════════
// layout_line_y
// ═══════════════════════════════════════════════════════════════════

#[test]
fn line_y_first_line() {
    let doc = shaped_multi("hello\nworld\nfoo");
    let y0 = layout_line_y(&doc, 0);
    assert_eq!(y0, 0.0);
}

#[test]
fn line_y_second_line() {
    let doc = shaped_multi("hello\nworld\nfoo");
    let y0 = layout_line_y(&doc, 0);
    let y1 = layout_line_y(&doc, 1);
    assert!(y1 > y0, "line 1 should be below line 0: y1={y1} y0={y0}");
}

#[test]
fn line_y_third_line() {
    let doc = shaped_multi("hello\nworld\nfoo");
    let y1 = layout_line_y(&doc, 1);
    let y2 = layout_line_y(&doc, 2);
    assert!(y2 > y1, "line 2 should be below line 1: y2={y2} y1={y1}");
}

#[test]
fn line_y_invalid_line() {
    let doc = shaped_multi("hello\nworld\nfoo");
    let y = layout_line_y(&doc, 99);
    assert_eq!(y, 0.0);
}

#[test]
fn line_y_empty_doc() {
    let doc = shaped_doc("", 200.0);
    let y = layout_line_y(&doc, 0);
    assert_eq!(y, 0.0);
}

#[test]
fn line_y_monotonic_increasing() {
    let doc = shaped_multi("a\nb\nc\nd\ne\nf\ng\nh\ni\nj");
    let mut prev = layout_line_y(&doc, 0);
    for i in 1..10 {
        let y = layout_line_y(&doc, i);
        assert!(y >= prev, "line {i} y={y} should be >= prev={prev}");
        prev = y;
    }
}

#[test]
fn line_y_line_height_consistency() {
    let doc = shaped_multi("line0\nline1\nline2");
    let y0 = layout_line_y(&doc, 0);
    let y1 = layout_line_y(&doc, 1);
    let h0 = doc.line_height(0);
    assert!((y1 - y0 - h0).abs() < 0.001, "y1-y0 should equal line_height");
}
