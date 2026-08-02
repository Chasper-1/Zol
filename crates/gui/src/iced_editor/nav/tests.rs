use super::*;
use editor::cache::DocumentCache;
use editor::cache::MarkupCache;
use editor::font;
use editor::layout::reveal::RevealCtx;
use editor::render;
use editor::render::ShapedDocument;
use editor::segment::{MarkerCategory, Segment, STYLE_BOLD, STYLE_PLAIN};
use editor::state::EditMode;
use editor::theme::EditorTheme;
use crate::iced_editor::EditorInner;

// ------------------------------------------------------------------
// helpers
// ------------------------------------------------------------------

fn shaped_line(text: &str, size: f32) -> ShapedDocument {
    font::init();
    let metrics = cosmic_text::Metrics::new(size, size * 1.4);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    render::build(
        &mut doc, text, &cache, EditMode::Source, &theme, size, 24.0, 0.0, None, None, None,
    );
    doc
}

fn make_inner(text: &str) -> EditorInner {
    font::init();
    EditorInner::new(text.to_string())
}

fn shaped_live(text: &str, cache: &DocumentCache, reveal: Option<&RevealCtx>) -> ShapedDocument {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 14.0 * 1.4);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let theme = EditorTheme::default();
    render::build(
        &mut doc, text, cache, EditMode::Live, &theme, 14.0, 24.0, 0.0, None, None, reveal,
    );
    doc
}

// Cache for `"**bold**"`: one Segment at raw_start=2, raw_end=6, left_marker_len=2.
fn cache_bold() -> DocumentCache {
    DocumentCache {
        lines: vec![MarkupCache {
            segments: vec![Segment {
                text: "bold".to_string(),
                style: STYLE_BOLD,
                left_marker_len: 2,
                right_marker_len: 0,
                raw_start: 2,
                raw_end: 6,
                category: MarkerCategory::Inline,
            }],
        }],
        block_of_line: vec![None],
    }
}

fn set_cursor_raw(inner: &EditorInner, raw: usize) {
    let mut doc = inner.doc.borrow_mut();
    doc.set_cursor_raw(raw);
}

// ═══════════════════════════════════════════════════════════════════
// cursor_x_on_line
// ═══════════════════════════════════════════════════════════════════

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
    let x5 = cursor_x_on_line(&doc, 0, 5);
    assert!(x1 > x0, "x1={x1} should be > x0={x0}");
    assert!(x5 >= x1, "x5={x5} should be >= x1={x1}");
}

#[test]
fn cursor_x_on_bold_text() {
    let doc = shaped_line("**bold**", 14.0);
    let x0 = cursor_x_on_line(&doc, 0, 0);
    let x_mid = cursor_x_on_line(&doc, 0, 4);
    let x_end = cursor_x_on_line(&doc, 0, 8);
    assert_eq!(x0, 0.0);
    assert!(x_mid > x0);
    assert!(x_end >= x_mid);
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
fn cursor_x_on_line_out_of_range() {
    let doc = shaped_line("hello", 14.0);
    let x = cursor_x_on_line(&doc, 99, 0);
    assert_eq!(x, 0.0);
}

#[test]
fn cursor_x_on_second_line() {
    let doc = shaped_line("ab\ncd", 14.0);
    assert_eq!(doc.line_count(), 2);
    let x = cursor_x_on_line(&doc, 1, 0);
    assert_eq!(x, 0.0);
}

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

#[test]
fn cursor_x_unicode_text() {
    let doc = shaped_line("émoji 👍", 14.0);
    let x0 = cursor_x_on_line(&doc, 0, 0);
    let x1 = cursor_x_on_line(&doc, 0, 2); // после 'é'
    let x2 = cursor_x_on_line(&doc, 0, 7); // после '👍' (4 bytes)
    assert!(x1 > x0, "cursor should advance past multi-byte char");
    assert!(x2 > x1, "cursor should advance past emoji");
}

#[test]
fn cursor_x_large_text() {
    let text = "a".repeat(200);
    let doc = shaped_line(&text, 14.0);
    let x = cursor_x_on_line(&doc, 0, 100);
    assert!(x > 0.0, "should have positive x at 100th char");
}

#[test]
fn cursor_x_with_markers_visible() {
    let doc = shaped_line("**bold**", 14.0);
    // В Source режиме маркеры видны, и курсор должен быть на 'b' (байт 2)
    let x_at_b = cursor_x_on_line(&doc, 0, 2);
    let x_at_0 = cursor_x_on_line(&doc, 0, 0);
    assert!(x_at_b > x_at_0, "cursor at 'b' should be right of cursor at start");
}

#[test]
fn cursor_x_all_spaces() {
    let doc = shaped_line("     ", 14.0);
    let x = cursor_x_on_line(&doc, 0, 0);
    let x_end = cursor_x_on_line(&doc, 0, 5);
    assert_eq!(x, 0.0);
    assert!(x_end > x);
}

// ═══════════════════════════════════════════════════════════════════
// raw_at_x_on_line
// ═══════════════════════════════════════════════════════════════════

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

#[test]
fn raw_at_x_at_mid_point() {
    let doc = shaped_line("hello", 14.0);
    let x_mid = cursor_x_on_line(&doc, 0, 2);
    let raw = raw_at_x_on_line(&doc, 0, x_mid, 0, 5);
    assert_eq!(raw, 2, "should round to nearest glyph at x={x_mid}");
}

#[test]
fn raw_at_x_between_glyphs() {
    let doc = shaped_line("hello", 14.0);
    let x1 = cursor_x_on_line(&doc, 0, 1);
    let x2 = cursor_x_on_line(&doc, 0, 2);
    let between = (x1 + x2) / 2.0;
    let raw = raw_at_x_on_line(&doc, 0, between, 0, 5);
    // Должен вернуть ближайший glyph.start
    assert!(raw == 1 || raw == 2, "between x1={x1} x2={x2}: raw={raw}");
}

#[test]
fn raw_at_x_line_out_of_range() {
    let doc = shaped_line("hello", 14.0);
    let raw = raw_at_x_on_line(&doc, 99, 0.0, 0, 5);
    assert_eq!(raw, 0);
}

#[test]
fn raw_at_x_line_end_equals_start() {
    let doc = shaped_line("", 14.0);
    // line_end == line_start для пустой строки
    let raw = raw_at_x_on_line(&doc, 0, 0.0, 0, 0);
    assert_eq!(raw, 0);
}

#[test]
fn raw_at_x_unicode() {
    let doc = shaped_line("émoji", 14.0);
    let x_at_0 = cursor_x_on_line(&doc, 0, 0);
    let x_at_2 = cursor_x_on_line(&doc, 0, 2);
    let raw = raw_at_x_on_line(&doc, 0, (x_at_0 + x_at_2) / 2.0, 0, 6);
    assert!(raw == 0 || raw == 2, "raw should snap to byte boundary near multi-byte char");
}

#[test]
fn raw_at_x_multiple_lines() {
    let doc = shaped_line("ab\ncd", 14.0);
    let raw_line1 = raw_at_x_on_line(&doc, 0, 9999.0, 0, 2);
    assert_eq!(raw_line1, 2, "last position on line 0");
    let raw_line2 = raw_at_x_on_line(&doc, 1, 9999.0, 3, 5);
    assert_eq!(raw_line2, 5, "last position on line 1");
}

// ═══════════════════════════════════════════════════════════════════
// move_vertical
// ═══════════════════════════════════════════════════════════════════

#[test]
fn move_vertical_to_next_line() {
    let inner = make_inner("line zero\nline one");
    set_cursor_raw(&inner, 5);
    let old_line = inner.doc.borrow().cursor.line();
    move_vertical(&inner, 1);
    let new_line = inner.doc.borrow().cursor.line();
    assert_eq!(old_line, 0);
    assert_eq!(new_line, 1);
}

#[test]
fn move_vertical_to_prev_line() {
    let inner = make_inner("line zero\nline one");
    set_cursor_raw(&inner, 10);
    move_vertical(&inner, 0);
    let new_line = inner.doc.borrow().cursor.line();
    assert_eq!(new_line, 0);
}

#[test]
fn move_vertical_to_first_line() {
    let inner = make_inner("a\nb");
    set_cursor_raw(&inner, 2);
    move_vertical(&inner, 0);
    assert_eq!(inner.doc.borrow().cursor.line(), 0);
}

#[test]
fn move_vertical_to_last_line() {
    let inner = make_inner("a\nb");
    set_cursor_raw(&inner, 0);
    move_vertical(&inner, 1);
    assert_eq!(inner.doc.borrow().cursor.line(), 1);
}

#[test]
fn move_vertical_to_same_line() {
    let inner = make_inner("line zero\nline one");
    set_cursor_raw(&inner, 3);
    move_vertical(&inner, 0);
    assert_eq!(inner.doc.borrow().cursor.line(), 0);
    assert_eq!(inner.doc.borrow().cursor.raw(), 3);
}

#[test]
fn move_vertical_sets_dirty() {
    let inner = make_inner("a\nb");
    inner.doc.borrow_mut().dirty = false;
    move_vertical(&inner, 1);
    assert!(inner.doc.borrow().dirty, "move_vertical should set dirty=true");
}

#[test]
fn move_vertical_sets_col_visual() {
    let inner = make_inner("aaa\nbbb");
    move_vertical(&inner, 1);
    assert!(inner.doc.borrow().cursor.col_visual() >= 0.0, "col_visual should be set");
}

#[test]
fn move_vertical_preserves_col_visual() {
    let inner = make_inner("hello world\nshort");
    set_cursor_raw(&inner, 5); // после "hello"
    move_vertical(&inner, 1);
    assert!(inner.doc.borrow().cursor.col_visual() >= 0.0);
}

#[test]
fn move_vertical_to_single_line_no_change() {
    let inner = make_inner("single line");
    move_vertical(&inner, 0);
    assert_eq!(inner.doc.borrow().cursor.line(), 0);
}

#[test]
fn move_vertical_beyond_last_line_falls_back() {
    // move_vertical с target_line вне документа использует line_bounds,
    // который возвращает None, unwrap_or((0,0)) → raw=0
    let inner = make_inner("a\nb\nc");
    set_cursor_raw(&inner, 4);
    move_vertical(&inner, 99);
    // Курсор сбрасывается на 0 (fallback при отсутствии line_bounds)
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn move_vertical_negative_line_clamps() {
    let inner = make_inner("a\nb\nc");
    set_cursor_raw(&inner, 4);
    // target_line = 0 (min)
    move_vertical(&inner, 0);
    assert_eq!(inner.doc.borrow().cursor.line(), 0);
}

#[test]
fn move_vertical_on_empty_doc() {
    let inner = make_inner("");
    move_vertical(&inner, 0);
    assert_eq!(inner.doc.borrow().cursor.line(), 0);
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn move_vertical_shorter_target_line() {
    let inner = make_inner("hello world\nshort");
    set_cursor_raw(&inner, 6); // после пробела, col_visual ~ x at 'w'
    let x_before = inner.doc.borrow().cursor.raw();
    move_vertical(&inner, 1); // "short" (5 chars)
    let new_raw = inner.doc.borrow().cursor.raw();
    assert_ne!(x_before, new_raw, "cursor should move to a different position");
    // raw должен быть ≤ длины второй строки
    let bounds = inner.doc.borrow().line_bounds(1).unwrap();
    assert!(new_raw <= bounds.end, "cursor raw {new_raw} should not exceed line end {}", bounds.end);
}

// ═══════════════════════════════════════════════════════════════════
// Navigation with compensation (hidden markers in Live preview)
// ═══════════════════════════════════════════════════════════════════

// ─── cursor_x_on_line with hidden markers ─────────────────────────

#[test]
fn cursor_x_hidden_markers_snaps_to_content_start() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    let x0 = cursor_x_on_line(&doc, 0, 0);
    let x1 = cursor_x_on_line(&doc, 0, 1);
    let x2 = cursor_x_on_line(&doc, 0, 2);
    assert_eq!(
        x0, x2,
        "byte 0 (inside **) should snap to same x as byte 2 (start of 'bold')"
    );
    assert_eq!(x1, x2, "byte 1 (inside **) should also snap");
}

#[test]
fn cursor_x_hidden_markers_content_progresses() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    let x_b = cursor_x_on_line(&doc, 0, 2);
    let x_o = cursor_x_on_line(&doc, 0, 3);
    let x_d = cursor_x_on_line(&doc, 0, 5);
    assert!(x_o > x_b, "'o' should be right of 'b'");
    assert!(x_d > x_o, "'d' should be right of 'o'");
}

#[test]
fn cursor_x_hidden_markers_snaps_to_content_end() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    let x6 = cursor_x_on_line(&doc, 0, 6);
    let x7 = cursor_x_on_line(&doc, 0, 7);
    let x8 = cursor_x_on_line(&doc, 0, 8);
    assert_eq!(x6, x8, "byte 6 (after 'd') should match byte 8 (end)");
    assert_eq!(x7, x8, "byte 7 (inside **) should snap to end");
}

// ─── raw_at_x_on_line with hidden markers ─────────────────────────

#[test]
fn raw_at_x_hidden_markers_click_on_content() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    let line_end = 8;
    let x_b = cursor_x_on_line(&doc, 0, 2);
    assert_eq!(raw_at_x_on_line(&doc, 0, x_b, 0, line_end), 2);
    let x_o = cursor_x_on_line(&doc, 0, 3);
    assert_eq!(raw_at_x_on_line(&doc, 0, x_o, 0, line_end), 3);
    let x_d = cursor_x_on_line(&doc, 0, 5);
    assert_eq!(raw_at_x_on_line(&doc, 0, x_d, 0, line_end), 5);
}

#[test]
fn raw_at_x_hidden_markers_end_of_line() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    assert_eq!(raw_at_x_on_line(&doc, 0, 9999.0, 0, 8), 8);
}

#[test]
fn raw_at_x_hidden_markers_negative_x_snaps_to_first_content() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    assert_eq!(raw_at_x_on_line(&doc, 0, -100.0, 0, 8), 2);
}

// ─── Roundtrip ────────────────────────────────────────────────────

#[test]
fn roundtrip_hidden_markers_content_positions() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    let line_end = 8;
    for byte_in_line in 2..6 {
        let x = cursor_x_on_line(&doc, 0, byte_in_line);
        let recovered = raw_at_x_on_line(&doc, 0, x, 0, line_end);
        assert_eq!(
            recovered, byte_in_line,
            "roundtrip failed for content byte={}",
            byte_in_line
        );
    }
}

#[test]
fn roundtrip_source_mode_with_marker_text() {
    let doc = shaped_line("**bold**", 14.0);
    let line_end = 8;
    for &byte_in_line in &[0, 2, 3, 4, 5, 6, 8] {
        let x = cursor_x_on_line(&doc, 0, byte_in_line);
        let recovered = raw_at_x_on_line(&doc, 0, x, 0, line_end);
        assert_eq!(
            recovered, byte_in_line,
            "source mode roundtrip failed for byte={}",
            byte_in_line
        );
    }
}

#[test]
fn roundtrip_live_plain_line_matches_source() {
    let cache = DocumentCache {
        lines: vec![MarkupCache {
            segments: vec![Segment {
                text: "hello".to_string(),
                style: STYLE_PLAIN,
                left_marker_len: 0,
                right_marker_len: 0,
                raw_start: 0,
                raw_end: 5,
                category: MarkerCategory::Inline,
            }],
        }],
        block_of_line: vec![None],
    };
    let live_doc = shaped_live("hello", &cache, None);
    let source_doc = shaped_line("hello", 14.0);
    for byte in 0..=5 {
        let x_live = cursor_x_on_line(&live_doc, 0, byte);
        let x_source = cursor_x_on_line(&source_doc, 0, byte);
        assert_eq!(
            x_live, x_source,
            "live should match source at byte={}: live={}, source={}",
            byte, x_live, x_source
        );
    }
    for &byte in &[0, 2, 5] {
        let x = cursor_x_on_line(&live_doc, 0, byte);
        let recovered = raw_at_x_on_line(&live_doc, 0, x, 0, 5);
        assert_eq!(recovered, byte, "live plain roundtrip failed at byte={}", byte);
    }
}

// ─── Edge cases ───────────────────────────────────────────────────

#[test]
fn live_mode_empty_line() {
    let cache = DocumentCache {
        lines: vec![MarkupCache { segments: vec![] }],
        block_of_line: vec![None],
    };
    let doc = shaped_live("", &cache, None);
    assert_eq!(cursor_x_on_line(&doc, 0, 0), 0.0);
    assert_eq!(raw_at_x_on_line(&doc, 0, 0.0, 0, 0), 0);
}

#[test]
fn markers_revealed_when_cursor_on_line() {
    let bo = vec![None];
    let ctx = RevealCtx {
        cursor_raw: Some(4),
        cursor_line: Some(0),
        block_of_line: &bo,
    };
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, Some(&ctx));
    let x0 = cursor_x_on_line(&doc, 0, 0);
    let x2 = cursor_x_on_line(&doc, 0, 2);
    assert!(x2 > x0, "revealed markers: 'b' should be right of '**'");
    assert_eq!(raw_at_x_on_line(&doc, 0, x0, 0, 8), 0);
    assert_eq!(raw_at_x_on_line(&doc, 0, x2, 0, 8), 2);
}

#[test]
fn compensation_mid_line_markers() {
    let content = "text **bold** more";
    let cache = DocumentCache {
        lines: vec![MarkupCache {
            segments: vec![
                Segment {
                    text: "text ".to_string(),
                    style: STYLE_PLAIN,
                    left_marker_len: 0,
                    right_marker_len: 0,
                    raw_start: 0,
                    raw_end: 5,
                    category: MarkerCategory::Inline,
                },
                Segment {
                    text: "bold".to_string(),
                    style: STYLE_BOLD,
                    left_marker_len: 2,
                    right_marker_len: 0,
                    raw_start: 7,
                    raw_end: 11,
                    category: MarkerCategory::Inline,
                },
                Segment {
                    text: " more".to_string(),
                    style: STYLE_PLAIN,
                    left_marker_len: 0,
                    right_marker_len: 0,
                    raw_start: 13,
                    raw_end: 18,
                    category: MarkerCategory::Inline,
                },
            ],
        }],
        block_of_line: vec![None],
    };
    let doc = shaped_live(content, &cache, None);
    let line_end = content.len();
    // Click on visual 'b' (after hidden **) → buffer position 7
    let x_b = cursor_x_on_line(&doc, 0, 7);
    assert_eq!(raw_at_x_on_line(&doc, 0, x_b, 0, line_end), 7);
    // Click on visual space after bold → buffer position 13
    let x_space = cursor_x_on_line(&doc, 0, 13);
    assert_eq!(raw_at_x_on_line(&doc, 0, x_space, 0, line_end), 13);
    // Roundtrip for all content positions
    for &byte in &[0, 1, 2, 3, 4, 7, 8, 9, 10, 13, 14, 15, 16, 17] {
        let x = cursor_x_on_line(&doc, 0, byte);
        let recovered = raw_at_x_on_line(&doc, 0, x, 0, line_end);
        assert_eq!(
            recovered, byte,
            "mid-line roundtrip failed at byte={}",
            byte
        );
    }
}

#[test]
fn compensation_multiline_hidden_markers() {
    // Line 0: plain "hello", line 1: "**bold**" with hidden markers
    let cache = DocumentCache {
        lines: vec![
            MarkupCache {
                segments: vec![Segment {
                    text: "hello".to_string(),
                    style: STYLE_PLAIN,
                    left_marker_len: 0,
                    right_marker_len: 0,
                    raw_start: 0,
                    raw_end: 5,
                    category: MarkerCategory::Inline,
                }],
            },
            MarkupCache {
                segments: vec![Segment {
                    text: "bold".to_string(),
                    style: STYLE_BOLD,
                    left_marker_len: 2,
                    right_marker_len: 0,
                    raw_start: 8,
                    raw_end: 12,
                    category: MarkerCategory::Inline,
                }],
            },
        ],
        block_of_line: vec![None, None],
    };
    let doc = shaped_live("hello\n**bold**", &cache, None);
    assert_eq!(doc.line_count(), 2);
    // Line 0: plain text, should still work
    let x0 = cursor_x_on_line(&doc, 0, 0);
    let x5 = cursor_x_on_line(&doc, 0, 5);
    assert!(x5 > x0);
    assert_eq!(raw_at_x_on_line(&doc, 0, x0, 0, 5), 0);
    assert_eq!(raw_at_x_on_line(&doc, 0, x5, 0, 5), 5);
    // Line 1: hidden markers, compensation active
    // byte_in_line is relative to line start. abs 8 → rel 2 ('b')
    let x_b = cursor_x_on_line(&doc, 1, 2);
    // abs 14 → rel 8 (past closing **)
    let x_end = cursor_x_on_line(&doc, 1, 8);
    // raw_at_x_on_line takes absolute line_start/line_end, returns absolute buffer pos
    assert_eq!(raw_at_x_on_line(&doc, 1, x_b, 6, 14), 8);
    assert_eq!(raw_at_x_on_line(&doc, 1, x_end, 6, 14), 14);
}

#[test]
fn raw_at_x_between_glyphs_hidden_markers() {
    let cache = cache_bold();
    let doc = shaped_live("**bold**", &cache, None);
    let x_b = cursor_x_on_line(&doc, 0, 2);
    let x_o = cursor_x_on_line(&doc, 0, 3);
    let between = (x_b + x_o) / 2.0;
    let raw = raw_at_x_on_line(&doc, 0, between, 0, 8);
    assert!(raw == 2 || raw == 3, "between 'b' and 'o': raw={}", raw);
}
