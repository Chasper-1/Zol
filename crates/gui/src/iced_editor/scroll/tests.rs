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
    let new_scroll = ensure_cursor_visible(50.0, 100.0, &doc, 0);
    assert!(new_scroll < 50.0, "should scroll up, got {new_scroll}");
    assert_eq!(new_scroll, 0.0);
}

#[test]
fn cursor_below_viewport() {
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
fn cursor_below_viewport_triggers_scroll() {
    let doc = shaped_doc("a\nb\nc\nline four", 200.0);
    let cursor_line = 3;
    let cursor_y = doc
        .buffer
        .layout_runs()
        .nth(cursor_line)
        .map(|r| r.line_top)
        .unwrap_or(0.0);
    let vp = cursor_y * 0.8;
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
